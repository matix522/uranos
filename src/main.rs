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

#[cfg(any(feature = "raspi3", feature = "raspi2"))]
const MMIO_BASE: u32 = 0x3E00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: u32 = 0xFE00_0000;

use io::UART;
///
/// # Safety 
/// Function should only be passed into entry! macro 
unsafe fn kernel_entry() -> ! {
    let mut mbox = mbox::Mbox::new();

    // let u = uart::Uart;

    // u.init(&mut mbox);
    // u.puts("PRE Mutex");
    // let uart = io::UART.lock();

    //set up serial console
    match UART.init(&mut mbox) {
        Ok(_) =>  UART.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
            // unsafe { 
                asm!("wfe" :::: "volatile") 
                // }; // If UART fails, abort early
        },
    }
    

    println!("{:?} {:?}", 5, 10);

    println!("Exception Level: {:?}", boot::mode::ExceptionLevel::get_current());
    // echo everything back

    loop { 
        UART.send(UART.getc());
    }
}
// pub unsafe fn kernel_entry() -> ! {
//     gpio::setup();
//     gpio::blink();
// }
boot::entry!(kernel_entry);