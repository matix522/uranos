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

use aarch64::halt;

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: u32 = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: u32 = 0xFE00_0000;

extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vectors_start: u64;
}

fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();
    let uart = uart::Uart::new();
    // let u = uart::Uart;

    // u.init(&mut mbox);
    // u.puts("PRE Mutex");
    // let uart = io::UART.lock();
        
    //set up serial console
    match uart.init(&mut mbox) {
        Ok(_) =>  println!("[ Ok ] UART is live!"),
        Err(_) => halt()  // If UART fails, abort early
        
    }

    
    uart.puts("Initializing IRQ_vector_table\n\r");
    unsafe { 
        
        let exception_vectors_start: u64 = &__exception_vectors_start as *const _ as u64;
        println!("vector table at {:x}", exception_vectors_start);
        interupt::set_vector_table_pointer(exception_vectors_start); 
    }
    interupt::daif_clr(2);
    interupt::timer::ArmQemuTimer::interupt_after(interupt::timer::ArmQemuTimer::get_frequency());
    interupt::timer::ArmQemuTimer::enable();

    println!("Exception Level: {:?}", boot::mode::ExceptionLevel::get_current());

        println!("time {}", interupt::timer::ArmQemuTimer::get_time());
        println!("to interupt {}", interupt::timer::ArmQemuTimer::ticks_to_interupt());

    unsafe{

        println!("Boot cores: {:x} ", _boot_cores as *const () as u64 );
        println!("reset: {:x} ",boot::reset as *const () as u64 );
        println!("kernel: {:x} ",kernel_entry as *const () as u64 );
    }

    // echo everything back
    loop { 
        println!("time {}", interupt::timer::ArmQemuTimer::get_time());
        println!("to interupt {}", interupt::timer::ArmQemuTimer::ticks_to_interupt());
        uart.send(uart.getc());
    }
}
// pub unsafe fn kernel_entry() -> ! {
//     gpio::setup();
//     gpio::blink();
// }
boot::entry!(kernel_entry);