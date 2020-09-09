pub mod allocator;
pub mod armv8;
pub mod memory_controler;
#[allow(dead_code)]
pub mod physical {
    #[cfg(feature = "raspi3")]
    #[rustfmt::skip]
    pub const MEMORY_END:          usize =        0x4000_1000;

    #[cfg(not(feature = "raspi3"))]
    #[rustfmt::skip]
    pub const MEMORY_END:          usize =        0xFFFF_FFFF;

    /// Physical devices.
    #[rustfmt::skip]
    pub mod mmio {
        #[cfg(feature = "raspi3")]
        pub const BASE:            usize =        0x3F00_0000;

        #[cfg(not(feature = "raspi3"))]
        pub const BASE:            usize =        0xFE00_0000;

        pub const GPIO_BASE:       usize = BASE + 0x0020_0000;
        pub const UART_BASE:       usize = BASE + 0x0020_1000;
        #[cfg(feature = "raspi3")]
        pub const END:             usize =        0x4020_0000;
        #[cfg(not(feature = "raspi3"))]
        pub const END:             usize =        0xFFFF_FFFF;

    }
}
