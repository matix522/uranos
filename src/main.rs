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

const INTERRUPT_CONTROLLER_BASE: usize = MMIO_BASE + 0xB200;
const kernel_offset : usize = 0xFFFF_0000_0000_0000usize | !((1 << 36) -1);

use drivers::traits::console::*;
use drivers::traits::Init;

use drivers::rpi3_interrupt_controller::Rpi3InterruptController;
use crate::interupts::interrupt_controller::InterruptController;
use drivers::rpi3_interrupt_controller::IRQType;
use utils::debug::printRegisterAddress;

use crate::time::Timer;
use time::arm::ArmTimer;

use core::ops::Deref;

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
    
    let controller = Rpi3InterruptController::new(INTERRUPT_CONTROLLER_BASE);
    
    #[cfg(feature = "debug")]
    printRegisterAddress(&controller.deref());

    interupts::enable_irqs();

    controller.disable_IRQ(IRQType::ArmTimer);

    ArmTimer::interupt_after(ArmTimer::get_frequency());
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

        let ptr = t_string.as_ptr();
        let size = t_string.bytes().len();

        crate::println!("{:#018x}", kernel_offset);

        let ptr = (kernel_offset | ptr as usize) as *const u8;
        let kernel_str = core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, size));

        crate::println!("ORGINAL {:#018x}: {}", t_string.as_ptr() as usize, t_string);
        crate::println!("MOVED   {:#018x}: {}", moved_str.as_ptr() as usize, moved_str);
        crate::println!("KERNEL  {:#018x}: {}", kernel_str.as_ptr() as usize, kernel_str);

    }
    jump_to_kernel_space(echo);

}
extern "C" fn jump_to_kernel_space(f : fn () -> !) -> ! {
    let address = f as * const () as u64;
    unsafe { llvm_asm!("brk 0" : : "{x0}"(address) : : "volatile"); }
    loop{};
}
fn echo () -> !{
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
