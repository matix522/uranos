#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
// extern crate spin;

pub mod gpio;
pub mod mbox;
pub mod uart;
pub mod io;
pub mod time;
pub mod interupt;
pub mod sync;

use aarch64::*;

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: u32 = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: u32 = 0xFE00_0000;

extern "C" {
    pub fn _boot_cores() -> !;
    static mut __binary_end : u64;
}

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    match uart.init(&mut mbox) {
        Ok(_) =>  println!("[ OK ] Uart Loaded"),
        Err(_) => halt()
    }
    
    println!("Exception Level: {:?}", boot::mode::ExceptionLevel::get_current());

    println!("Binary loaded at: {:x} ", _boot_cores as *const () as u64 );

    println!("Binary ends at: {:x} ", unsafe { __binary_end } );

    println!("Kernel Initialization complete.");

    // echo everything back
    
    loop { 
        uart.send(uart.getc());
    }
}

boot::entry!(kernel_entry);