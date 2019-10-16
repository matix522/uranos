use crate::uart::*;
use core::fmt;
pub static mut UART: Uart = Uart::new(); //Mutex::new(Uart); //= Uart::new();

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.puts(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::print!("\x1b[31m{}\x1b[0m", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        UART.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! scanln {
    ($( $x:ty ),+ ) => {{
        print!("\x1B[38;2;100;255;255m(No Filesystem  )/ \x1B[38;2;200;255;100m❯\x1B[0m");
        let res;

        let stdio = kernel::get_kernel_ref().get_stdio();
        res = stdio.get_line();

        let string = core::str::from_utf8( &res.1).unwrap();
        let mut iter = string.split_ascii_whitespace();
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    }}
}
