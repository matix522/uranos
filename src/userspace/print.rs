
use crate::syscall::print::print;
struct SyscallPrint;

impl core::fmt::Write for SyscallPrint {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        print(s);
        Ok(())
    }
}
#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => ($crate::userspace::print::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! uprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::uprint!("{}\n", format_args!($($arg)*)));
}
#[allow(unused_macros)]
macro_rules! euprint {
    ($($arg:tt)*) => ($crate::uprint!("\x1b[31m{}\x1b[0m", format_args!($($arg)*)));
}
#[allow(unused_macros)]
macro_rules! euprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::euprint!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SyscallPrint.write_fmt(args).expect("Print Syscall Error")
}

