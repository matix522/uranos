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
pub mod interupts;
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
    pub static __exception_vector_start: u64;
    pub static __binary_end: u64;
    pub static __read_only_start: usize;
    pub static __read_only_end: usize;
}

#[derive(Debug)]
struct BinaryInfo {
    binary_start : usize,
    binary_end : usize,
    read_only_start : usize,
    read_only_end : usize,
    exception_vector : usize,
    heap_start : usize,
    heap_end : usize,
}
impl BinaryInfo {
    fn get () -> BinaryInfo {
        unsafe {
            BinaryInfo {
                binary_start : _boot_cores as *const () as usize,
                binary_end : &__binary_end as *const _ as usize,
                read_only_start : &__read_only_start as *const _ as usize,
                read_only_end : &__read_only_end as *const _ as usize,
                exception_vector : &__exception_vector_start as *const _ as usize,
                heap_start : memory::allocator::heap_start(),
                heap_end : memory::allocator::heap_end(),
            }
        }
    }
}
use core::fmt;
impl fmt::Display for BinaryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> { 
        use memory::allocator::{heap_start, heap_end};
        writeln!(f, "Binary definition:")?;
        writeln!(f, "\tRange:            [{:#10x}  {:#10x}]", self.binary_start, self.binary_end)?;
        writeln!(f, "\tRead Only Range:  [{:#10x}  {:#10x}]", self.read_only_start, self.read_only_end)?;
        writeln!(f, "\tException Vector: [{:#10x}            ]", self.exception_vector)?;
        writeln!(f, "\tMain Heap:        [{:#10x}  {:#10x}]", heap_start(), heap_end())?;
        Ok(())
    }
}


use drivers::traits::console::*;
use drivers::traits::Init;

fn kernel_entry() -> ! {
    let uart = drivers::UART.lock();
    match uart.init() {
        Ok(_) => println!("\x1B[2J\x1B[2;1H[ Ok ] UART is live!"),
        Err(_) => halt(), // If UART fails, abort early
    }
    drop(uart);

    println!("{}", BinaryInfo::get());
    
    
    unsafe {
        interupts::init_exceptions();
    }
    let big_addr: u64 = 1024 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    println!("Kernel Initialization complete.");
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
