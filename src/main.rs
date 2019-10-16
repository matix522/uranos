#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]

// extern crate spin;
extern crate alloc;

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



// pub fn init_f(){
//     loop {
//         println!("Hello from init task!");
//         for i in 1..10000 {
//             unsafe{asm!{"nop" :::: "volatile"}}
//         }
//     }
// }


// pub fn test_task_f(){
//     loop {
//         println!("Hello from test task!");
//         for i in 1..10000 {
//             unsafe{asm!{"nop" :::: "volatile"}}
//         }
//     }
// }

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

    uart.puts("Initializing IRQ_vector_table\n\r");
    unsafe {
        let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;
        println!("vector table at {:x}", exception_vectors_start);
        interupt::set_vector_table_pointer(exception_vectors_start);
    }

    println!("Binary loaded at: {:x} ", _boot_cores as *const () as u64);

    println!("Binary ends at: {:x} ", unsafe {
        &__binary_end as *const u64 as u64
    });

    println!("Kernel Initialization complete.");

    // echo everything back

    // let init : scheduler::TaskContext = scheduler::TaskContext::new(init_f, 0);
    // let another : scheduler::TaskContext = scheduler::TaskContext::new(test_task_f, 0);

    // init.start();
    // another.start();

    // scheduler::schedule_first();

    loop {
        uart.send(uart.getc());
    }
}

boot::entry!(kernel_entry);
