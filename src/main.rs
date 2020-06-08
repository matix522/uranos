#![no_std]
#![no_main]
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(never_type)]
#![feature(inner_deref)]
#![feature(const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(crate_visibility_modifier)]
#![feature(panic_info_message)]
#![feature(concat_idents)]
#![allow(incomplete_features)]

extern crate alloc;
extern crate num_derive;
extern crate static_assertions;
pub mod drivers;

pub mod aarch64;
pub mod boot;
pub mod io;
pub mod memory;

pub mod sync;
pub mod time;

pub mod utils;

use core::panic::PanicInfo;

use aarch64::*;

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: usize = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: usize = 0xFE00_0000;

extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vectors_start: u64;
    pub static __binary_end: u64;
    pub static __read_only_start: usize;
    pub static __read_only_end: usize;
}

use drivers::traits::Init;
use drivers::traits::console::*;

fn kernel_entry() -> ! {
    let uart = drivers::UART.lock();
    match uart.init() {
        Ok(_) => println!("\x1B[2J\x1B[2;1H[ Ok ] UART is live!"),
        Err(_) => halt(), // If UART fails, abort early
    }
    drop(uart);


    println!("TESTING");
 
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


    println!("Kernel Initialization complete.");
    let gpio = drivers::GPIO.lock();
    println!("GPFSEL1 {:x}", &gpio.GPFSEL1 as *const _ as u64);
    println!("GPFSEL2 {:x}", &gpio.GPFSEL2 as *const _ as u64);
    println!("GPSET0 {:x}", &gpio.GPSET0 as *const _ as u64);
    println!("GPCLR0 {:x}", &gpio.GPCLR0 as *const _ as u64);
    println!("GPPUD {:x}", &gpio.GPPUD as *const _ as u64);
    println!("GPPUDCLK0 {:x}", &gpio.GPPUDCLK0 as *const _ as u64);
    drop(gpio);

    let uart = drivers::UART.lock();
    let echo_loop = || -> Result<!, &str> {
        loop {
            uart.putc(uart.getc()?);
        }
    };
    loop {
        let value = echo_loop().unwrap_err();
        println!("{}", value);
    }
}

entry!(kernel_entry);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(args) = info.message() {
        println!("\nKernel panic: {}", args);
    } else {
        println!("\nKernel panic!");
    }
    halt();
}
