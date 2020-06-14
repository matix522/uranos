pub trait Init {
    fn init(&self) -> Result<(), &'static str>;
}

pub mod console {
    pub trait Read {
        /// read byte character
        fn getb(&self) -> u8;
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
