#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]

// extern crate spin;
extern crate alloc;

pub mod gpio;
pub mod mbox;
pub mod uart;
pub mod io;
pub mod time;
pub mod interupt;
pub mod sync;
pub mod memory;

use aarch64::*;

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: u32 = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: u32 = 0xFE00_0000;

extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vectors_start: u64;
    static mut __binary_end : u64;
}

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();

    match uart.init(&mut mbox) {
        Ok(_) =>  println!("[ Ok ] UART is live!"),
        Err(_) => halt()  // If UART fails, abort early
    }

    println!("Exception Level: {:?}", boot::mode::ExceptionLevel::get_current());

        uart.puts("Initializing IRQ_vector_table\n\r");
    unsafe { 
        
        let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;
        println!("vector table at {:x}", exception_vectors_start);
        interupt::set_vector_table_pointer(exception_vectors_start); 
    }
    interupt::daif_clr(2);
    interupt::timer::ArmQemuTimer::interupt_after(interupt::timer::ArmQemuTimer::get_frequency());
    interupt::timer::ArmQemuTimer::enable();

    println!("Binary loaded at: {:x} ", _boot_cores as *const () as u64 );

    println!("Binary ends at: {:x} ", unsafe { __binary_end } );

    println!("Kernel Initialization complete.");

    // echo everything back
    
    loop { 
        println!("time {}", interupt::timer::ArmQemuTimer::get_time());
        println!("to interupt {}", interupt::timer::ArmQemuTimer::ticks_to_interupt());
        uart.send(uart.getc());
    }
}

boot::entry!(kernel_entry);