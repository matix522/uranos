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
pub mod framebuffer;

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

fn kernel_entry() -> ! {
    let mut mbox = drivers::MBOX.lock();
    let uart = drivers::UART.lock();
    match uart.init(&mut mbox) {
        Ok(_) => println!("\x1B[2J\x1B[2;1H[ Ok ] UART is live!"),
        Err(_) => halt(), // If UART fails, abort early
    }
    drop(uart);
    drop(mbox);

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

    // print!("Initializing Interupt Vector Table: ");
    // unsafe {
    //     let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;
    //     println!("{:x}", exception_vectors_start);
    //     interupt::set_vector_table_pointer(exception_vectors_start);

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
    loop {
        uart.putc(uart.getc());
    }

    // println!("Proceeding init task initialization");

    // let init_task = scheduler::TaskContext::new(init::init, 1, true).unwrap();
    // println!("Init task created");
    // // println!("{:?}",init_task);
    // init_task.start_task().unwrap();

    // println!("Init task created and started");
    // let another_task = scheduler::TaskContext::new(init::test_task, 1, true);

    // another_task.start_task().unwrap();
    // println!("Another_task created");
    // let another_task2 = scheduler::TaskContext::new(init::test_task2, 1, false);

    // another_task2.start_task().unwrap();
    // println!("Another_task2 created");

    // if cfg!(feature = "raspi4") {
    //     use interupt::InteruptController;
    //     let mut gicv2 = interupt::gicv2::GICv2 {};
    //     gicv2.init().unwrap();
    // }

    // println!("freq: {}", Timer::get_frequency());

    // interupt::enable_irqs();
    // Timer::interupt_after(Timer::get_frequency() / 100);
    // Timer::enable();
    // println!("Timer enabled");

    // println!("time: {}", Timer::get_time());

    // scheduler::start();
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
