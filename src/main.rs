#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(never_type)]
#![feature(inner_deref)]
#![feature(const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(crate_visibility_modifier)]
// extern crate spin;
extern crate alloc;
#[macro_use]
extern crate num_derive;

pub mod framebuffer;
pub mod gpio;

pub mod interupt;
pub mod io;
pub mod mbox;
pub mod memory;
/// Task scheduler
pub mod scheduler;
pub mod sync;
pub mod time;
pub mod uart;
pub mod userspace;

pub mod utils;

mod init;

use interupt::timer::ArmQemuTimer as Timer;
pub mod devices;

use aarch64::*;

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: u32 = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: u32 = 0xFE00_0000;

extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vectors_start: u64;
    pub static __binary_end: u64;
    pub static __read_only_start: usize;
    pub static __read_only_end: usize;
}

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    match uart.init(&mut mbox) {
        Ok(_) => println!("\x1B[2J\x1B[2;1H[ Ok ] UART is live!"),
        Err(_) => halt(), // If UART fails, abort early
    }

    // let mut framebuffer = match framebuffer::FrameBuffer::new(&mut mbox) {
    //     Ok(framebuffer) => {
    //         println!("HDMI OK");
    //         framebuffer
    //     }
    //     Err(_) => {
    //         println!("HDMI FAILED");
    //         halt();
    //     }
    // };

    // use framebuffer::charbuffer::CharBuffer;
    // let framebuffer = framebuffer.as_mut().unwrap();
    // let charbuffer = CharBuffer::new(framebuffer);
    println!(
        "Exception Level: {:?}",
        boot::mode::ExceptionLevel::get_current()
    );

    println!(
        "Binary loaded at: {:x} - {:x}",
        _boot_cores as *const () as u64,
        unsafe { &__binary_end as *const u64 as u64 }
    );
    println!("Read only data ended at: {:x}", unsafe {
        &__read_only_end as *const usize as u64
    });
    println!(
        "Init Task Stack: {:x} - {:x}",
        _boot_cores as *const () as u64, 0
    );
    println!(
        "Main Heap: {:x} - {:x}",
        memory::allocator::heap_start(),
        memory::allocator::heap_end()
    );

    unsafe {
        use cortex_a::barrier;
        use cortex_a::regs::*;

        if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported) {
            crate::println!("64 KiB translation granule not supported");
        }

        memory::setup_mair();
        memory::setup_transaltion_tables();

        // Set the "Translation Table Base Register".
        TTBR0_EL1.set_baddr(memory::get_translation_table_address());

        memory::configure_translation_control();
        crate::println!("XDD");
        // Switch the MMU on.
        //
        // First, force all previous changes to be seen before the MMU is enabled.
        barrier::isb(barrier::SY);

        // Enable the MMU and turn on data and instruction caching.
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

        // Force MMU init to complete before next instruction.
        barrier::isb(barrier::SY);
        crate::println!("XDD");
    }
    print!("Initializing Interupt Vector Table: ");
    unsafe {
        let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;
        println!("{:x}", exception_vectors_start);
        interupt::set_vector_table_pointer(exception_vectors_start);
    }
    println!("Kernel Initialization complete.");

    println!("Proceeding init task initialization");

    let init_task = scheduler::TaskContext::new(init::init, 1, true).unwrap();
    println!("Init task created");
    // println!("{:?}",init_task);
    init_task.start_task().unwrap();

    println!("Init task created and started");
    // let another_task = scheduler::TaskContext::new(init::test_task, 1, true);

    // another_task.start_task().unwrap();
    // println!("Another_task created");
    // let another_task2 = scheduler::TaskContext::new(init::test_task2, 1, false);

    // another_task2.start_task().unwrap();
    // println!("Another_task2 created");

    if cfg!(feature = "raspi4") {
        use interupt::InteruptController;
        let mut gicv2 = interupt::gicv2::GICv2 {};
        gicv2.init().unwrap();
    }

    println!("freq: {}", Timer::get_frequency());

    interupt::enable_irqs();
    Timer::interupt_after(Timer::get_frequency() / 100);
    Timer::enable();
    println!("Timer enabled");

    println!("time: {}", Timer::get_time());

    scheduler::start();
    halt();
}

boot::entry!(kernel_entry);
