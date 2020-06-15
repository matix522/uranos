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
#![feature(new_uninit)]

extern crate alloc;
extern crate num_derive;
extern crate static_assertions;
pub mod drivers;

pub mod aarch64;
pub mod boot;
pub mod interupts;
pub mod io;
pub mod memory;

pub mod sync;
pub mod time;

pub mod utils;

use core::panic::PanicInfo;

use aarch64::*;
use utils::binary_info;

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: usize = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: usize = 0xFE00_0000;

const INTERRUPT_CONTROLLER_BASE: usize = 0x7E00_B000;

use drivers::traits::console::*;
use drivers::traits::Init;

use drivers::rpi3InterruptController::Rpi3InterruptController;

use crate::time::Timer;
use time::arm::ArmTimer;

fn kernel_entry() -> ! {
    let uart = drivers::UART.lock();
    match uart.init() {
        Ok(_) => println!("\x1B[2J\x1B[2;1H\x1B[2J\x1B[2;1H[ Ok ] UART is live!"),
        Err(_) => halt(), // If UART fails, abort early
    }
    drop(uart);
    let binary_info = binary_info::BinaryInfo::get();
    println!("{}", binary_info);

    unsafe {
        interupts::init_exceptions(binary_info.exception_vector);
    }

    println!("Enabling ARM Timer");
    
    interupts::enable_irqs();

    Rpi3InterruptController::disable_IRQ();

    ArmTimer::interupt_after(1000);
    ArmTimer::enable();

    println!("Kernel Initialization complete.");
    unsafe {
        println!("TEST mmu");

        let _ = memory::armv8::mmu::test();

        let t_string: &'static str = "Hello String";
        let ptr = t_string.as_ptr();
        let size = t_string.bytes().len();
        let ptr = ptr.add(0x1_0000_0000);
        let moved_str = core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, size));
        crate::println!("ORGINAL {}", t_string);
        crate::println!("MOVED {}", moved_str);

        // let address = 0x1_0000_0000 as *const u64;
        // crate::println!("{}", *address);
    }
    println!("Echoing input.");

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
