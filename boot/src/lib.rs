#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![feature(global_asm)]
#![feature(asm)]

//! Low-level boot of the Raspberry's processor

extern crate panic_abort;
extern crate aarch64;
/// Module contains code for 
pub mod mode;

/// Type check the user-supplied entry function.
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "main"]
        pub unsafe fn __main() -> ! {
            // type check the given path
            let f: unsafe fn() -> ! = $path;

            f()
        }
    };
}

/// Reset function.
///
/// Initializes the bss section before calling into the user's `main()`.
#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    extern "C" {
        // Boundaries of the .bss section, provided by the linker script
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    // Zeroes the .bss section
    r0::zero_bss(&mut __bss_start, &mut __bss_end);

    extern "Rust" {
        fn main() -> !;
    }
    mode::ExceptionLevel::drop_to_el1(main);   
}
// Disable all cores except core 0, and then jump to reset()
global_asm!(include_str!("boot.S"));