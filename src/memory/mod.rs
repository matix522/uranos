pub mod allocator;


use cortex_a::regs::*;


mod physical {
    #[cfg(feature = "raspi3")]
    #[rustfmt::skip]
    pub const MEMORY_END:          usize =        0x4100_0000;

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
        pub const END:             usize =        0x4100_0000;
        #[cfg(not(feature = "raspi3"))]
        pub const END:             usize =        0xFFFF_FFFF;

    }
    pub const fn address_space_size() -> usize {
        return MEMORY_END + 1;
    }
}

trait BaseAddr<U> {
    fn base_addr(&self) -> U;
}

impl<T, const N: usize> BaseAddr<u64> for [T; N] {
    fn base_addr(&self) -> u64 {
        self as *const T as u64
    }
}
