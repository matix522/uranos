use crate::drivers::uart::*;
use crate::drivers::UART;
use alloc::collections::VecDeque;
use core::fmt;

impl fmt::Write for PL011Uart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        use crate::drivers::traits::console::Write;
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
    UART.lock().write_fmt(args).expect("UART_WRITE_FMT")
}

#[macro_export]
macro_rules! scanln {
    ($( $x:ty ),+ ) => {{
        print!("\x1B[38;2;100;255;255m(No Filesystem  )/ \x1B[38;2;200;255;100m❯\x1B[0m");
        let res;

        let stdio = kernel::get_kernel_ref().get_stdio();
        res = stdio.get_line();

        let string = core::str::from_utf8( &res.1).expect("SCANLN");
        let mut iter = string.split_ascii_whitespace();
        ($(iter.next().and_then(|word| word.parse::<$x>().ok()),)*)
    }}
}
use crate::sync::nulllock::NullLock;

device_driver!(
    unsynchronized INPUT_BUFFER: VecDeque<u8> = VecDeque::new()
);

pub fn input_to_buffer() {
    use crate::drivers::traits::console::Read;
    let mut buffer = INPUT_BUFFER.lock();
    let uart = UART.lock();
    // crate::println!("AAA");
    // crate::println!("Buffer: {:x}", &*buffer as *const VecDeque<u8> as u64);

    while let Some(b) = uart.try_getb() {
        crate::println!("byte: {}", b);

        buffer.push_back(b);
    }
}

pub fn read_input(my_buffer: &mut [u8]) -> usize {
    let mut buffer = INPUT_BUFFER.lock();

    let count = core::cmp::min(buffer.len(), my_buffer.len());
    for i in 0..count {
        my_buffer[i] = buffer.pop_front().expect("should not happen");
    }
    count
}
