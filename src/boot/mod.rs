//! Low-level boot of the Raspberry Pi Cortex A53 and A72 processor

/// Module contains code for
pub mod mode;

/// Type check the user-supplied entry function.
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "main"]
        ///
        /// # Safety
        /// Function calling main rust kernel
        pub unsafe fn __main() -> ! {
            // type check the given path
            let f: fn() -> ! = $path;

            f()
        }
    };
}

/// Reset function.
///
/// Initializes the bss section before calling into the user's `main()`.
///
/// # Safety
/// Function Must be called by statup assembly code by only one core
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
