#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(never_type)]
// extern crate spin;
extern crate alloc;

pub mod gpio;
pub mod interupt;
pub mod io;
pub mod mbox;
pub mod memory;
pub mod userspace;
/// Task scheduler
pub mod scheduler;
pub mod sync;
pub mod time;
pub mod uart;

pub mod devices;

use aarch64::*;

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: u32 = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: u32 = 0xFE00_0000;

extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vectors_start: u64;
    static mut __binary_end: u64;
}

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    match uart.init(&mut mbox) {
        Ok(_) => println!("\x1B[2J\x1B[2;1H[ Ok ] UART is live!"),
        Err(_) => halt(), // If UART fails, abort early
    }

    println!(
        "Exception Level: {:?}",
        boot::mode::ExceptionLevel::get_current()
    );

    println!(
        "Binary loaded at: {:x} - {:x}",
        _boot_cores as *const () as u64,
        unsafe { &__binary_end as *const u64 as u64 }
    );
    println!(
        "Init Task Stack: {:x} - {:x}",
        _boot_cores as *const () as u64, 0
    );
    println!(
        "Main Heap: {:x} - {:x}",
        memory::allocator::heap_start(),
        memory::allocator::heap_end()
    );

    print!("Initializing Interupt Vector Table: ");
    unsafe {
        let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;
        println!("{:x}", exception_vectors_start);
        interupt::set_vector_table_pointer(exception_vectors_start);
    }
    // use interupt::timer::ArmQemuTimer as Timer;
    // interupt::daif_clr(2);
    // Timer::interupt_after(Timer::get_frequency());
    // Timer::enable();

    println!("Kernel Initialization complete.");

    println!("Proceeding init task initialization");
    let init_task = scheduler::TaskContext::new(scheduler::init::init, 1, true);
    println!("Init task created");
    // println!("{:?}",init_task);
    init_task.start_task().unwrap();
    println!("Init task created and started");
    let another_task = scheduler::TaskContext::new(scheduler::init::test_task, 2, false);

    another_task.start_task().unwrap();
    println!("Another_task created");
    let another_task2 = scheduler::TaskContext::new(scheduler::init::test_task2, 1, false);

    another_task2.start_task().unwrap();
    println!("Another_task2 created");

    let another_task3 = scheduler::TaskContext::new(scheduler::init::test_task2, 1, false);

    another_task3.start_task().unwrap();
    //use interupt::Inter
    if cfg!(feature = "raspi4") {
        use interupt::InteruptController;
        let mut gicv2 = interupt::gicv2::GICv2 {};
        gicv2.init().unwrap();
    }

    use interupt::timer::ArmQemuTimer as Timer;
    println!("freq: {}", Timer::get_frequency());

    interupt::enable_irqs();
    Timer::interupt_after(Timer::get_frequency());
    Timer::enable();
    println!("Timer enabled");
    // loop {
    //     uart.send(uart.getc());
    // }
    println!("time: {}", Timer::get_time());
    // let x = unsafe {
    //     userspace::syscall::syscall0(0)
    // };

    // println!("{}", x);

    match scheduler::start_scheduling(scheduler::init::init) {
        Ok(_) => loop{}, 
        Err(_) => halt()
    };

}

boot::entry!(kernel_entry);
