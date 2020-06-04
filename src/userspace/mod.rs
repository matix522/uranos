pub mod mutex;
pub mod syscall;
pub use num_traits::FromPrimitive;

use core::fmt;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum Syscalls {
    Print,
    NewTask,
    TerminateTask,
    GetTime,
    GetFrequency,
    Yield,
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
    SyscallWrite.write_fmt(args).unwrap();
}
#[macro_export]
macro_rules! uprint {
    ($($arg:tt)*) => ($crate::userspace::_uprint(format_args!($($arg)*)));
}
pub static MUTEX: crate::userspace::mutex::Mutex<()> = crate::userspace::mutex::Mutex::new(());
#[macro_export]
macro_rules! uprintln {
    () => ($crate::uprint!("\n"));
    ($($arg:tt)*) => ( $crate::userspace::MUTEX.sync(|_| $crate::uprint!("{}\n", format_args!($($arg)*))));
}
