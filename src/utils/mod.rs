pub mod binary_info;
pub mod debug;
pub fn delay(ticks: usize) {
    for _ in 0..ticks {
        crate::aarch64::asm::nop();
    }
}
pub fn utf8_char_width(first_byte: u8) -> usize {
    UTF8_CHAR_WIDTH[first_byte as usize] as usize
}
#[rustfmt::skip]
static UTF8_CHAR_WIDTH: [u8; 256] = [
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

pub mod color {
    #[derive(Clone, Copy)]
    pub struct RGBA {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: u8,
    }
    impl RGBA {
        pub const fn new(r: u8, g: u8, b: u8, a: u8) -> RGBA {
            RGBA { r, g, b, a }
        }
    }

    pub static BLACK: RGBA = RGBA::new(0, 0, 0, 255);
    pub static WHITE: RGBA = RGBA::new(255, 255, 255, 255);
}
