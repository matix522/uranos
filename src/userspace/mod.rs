pub mod syscall;
pub use num_traits::FromPrimitive;

use core::fmt;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum Syscalls {
    Print,
    NewTask,
}


struct SyscallWrite;
impl fmt::Write for SyscallWrite {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        syscall::write(s);
        Ok(())
    }
}
pub fn _uprint(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        SyscallWrite.write_fmt(args).unwrap();
    }
}
#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => ($crate::userspace::_uprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! uprintln {
    () => ($crate::uprint!("\n"));
    ($($arg:tt)*) => ($crate::uprint!("{}\n", format_args!($($arg)*)));
}
