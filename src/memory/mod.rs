pub mod allocator;
// pub mod mmu;
use core::mem::MaybeUninit;

mod physical {
    #[cfg(feature = "raspi3")]
    #[rustfmt::skip]
    pub const MEMORY_END:          usize =        0x3FFF_FFFF;

    #[cfg(feature = "raspi4")]
    #[rustfmt::skip]
    pub const MEMORY_END:          usize =        0xFFFF_FFFF;

    /// Physical devices.
    #[rustfmt::skip]
    pub mod mmio {
        #[cfg(feature = "raspi3")]
        pub const BASE:            usize =        0x3F00_0000;

        #[cfg(feature = "raspi4")]
        pub const BASE:            usize =        0xFE00_0000;

        pub const GPIO_BASE:       usize = BASE + 0x0020_0000;
        pub const UART_BASE:       usize = BASE + 0x0020_1000;
        pub const END:             usize =  super::MEMORY_END;
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct TableDescriptor(u64);

#[repr(C, align(4096))]
pub struct TopLevelTables {
    level_3_table : [TableDescriptor; 512], 
    level_2_tables : [[TableDescriptor;512];4],  // 4GB of space
}

static mut level_3_table : TopLevelTables = 
        TopLevelTables{level_3_table : [TableDescriptor(0);512], level_2_tables : [[TableDescriptor(0); 512]; 4]};

pub unsafe fn setup_transaltion_tables(kernel_range : (usize,usize), device_range : (usize,usize)){
    let allign_to_page = |x : usize| {
        if x % 4096 == 0 {
            x
        } else {
            (x / 4096 + 1) * 4096
        }
    };
}