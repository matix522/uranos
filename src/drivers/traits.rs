pub trait Init {
    fn init(&self) -> Result<(), &'static str>;
}
pub mod time {
    use core::time::Duration;
    pub trait Timer {
        fn get_time_raw(&self) -> u64;
        fn get_time(&self) -> Duration;
        fn interupt_after_raw(&self, ticks: u32);
        fn interupt_after(&self, time: Duration);
        fn enable(&self);
        fn disable(&self);
        fn get_frequency(&self) -> u32;
        fn wait_raw(&self, time: u64);
        fn wait(&self, time: Duration);
    }
}
pub mod console {
    pub trait Read {
        /// read byte character
        fn try_getb(&self) -> Option<u8>;

        fn getb(&self) -> u8 {
            loop {
                if let Some(t) = self.try_getb() {
                    return t;
                }
            }
        }
        /// read byte UTF-8 character
        fn getc(&self) -> Result<char, &'static str> {
            let first_byte = self.getb();

            let width = crate::utils::utf8_char_width(first_byte);
            if width == 1 {
                return Ok(first_byte as char);
            }
            if width == 0 {
                return Err("NotUtf8");
            }
            let mut buf = [first_byte, 0, 0, 0];
            {
                let mut start = 1;
                while start < width {
                    buf[start] = self.getb();
                    start += 1;
                }
            }
            match core::str::from_utf8(&buf[..width]) {
                Ok(s) => Ok(s.chars().next().unwrap()),
                Err(_) => Err("NotUtf8"),
            }
        }
    }
    pub trait Write {
        // Display a byte character
        fn putb(&self, b: u8);
        /// Display a UTF-8 character
        fn putc(&self, c: char) {
            let mut bytes: [u8; 4] = [0; 4];
            let _ = c.encode_utf8(&mut bytes);
            for b in &bytes {
                self.putb(*b);
            }
        }
        /// Display a string
        fn puts(&self, string: &str) {
            for b in string.chars() {
                // convert newline to carrige return + newline
                if b == '\n' {
                    self.putc('\r')
                }
                self.putc(b);
            }
        }
    }
}
