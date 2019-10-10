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

#[cfg(not(feature = "raspi4"))]
const MMIO_BASE: u32 = 0x3F00_0000;
#[cfg(feature = "raspi4")]
const MMIO_BASE: u32 = 0xFE00_0000;

extern "C" {
    pub fn _boot_cores() -> !;
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
        Ok(_) =>  uart.puts("\n[0] UART is live!\n"),
        Err(_) => loop {
             unsafe { 
                asm!("wfe" :::: "volatile") 
                 }; // If UART fails, abort early
        },
    }
    
    uart.puts("HELLO PIOTREK");
    uart.getc();
    uart.puts("\n\rXDDDDD");
    // println!("{:?} {:?}", 5, 10);

    // println!("Exception Level: {:?}", boot::mode::ExceptionLevel::get_current());
    // // echo everything back
    


    unsafe{

        println!("Boot cores: {:x} ", _boot_cores as *const () as u64 );
        println!("reset: {:x} ",boot::reset as *const () as u64 );
        println!("kernel: {:x} ",kernel_entry as *const () as u64 );
    }
    loop { 
        uart.send(uart.getc());
    }
}
// pub unsafe fn kernel_entry() -> ! {
//     gpio::setup();
//     gpio::blink();
// }
boot::entry!(kernel_entry);