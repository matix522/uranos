pub mod allocator;
pub mod page_descriptor;
pub mod table_descriptor;
pub mod attributes;

pub use page_descriptor::*;
pub use table_descriptor::*;
pub use attributes::*;
mod physical {
    #[cfg(feature = "raspi3")]
    #[rustfmt::skip]
    pub const MEMORY_END:          usize =        0x3FFF_FFFF;

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
        pub const END:             usize =  super::MEMORY_END;
    }
    pub const fn address_space_size() -> usize {
        return MEMORY_END +1 ;
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

#[repr(C, align(4096))]
pub struct TopLevelTables<const N : usize> {
    level_3: [[PageDescriptor; 8192]; N], // 512 MB each
    level_2: [TableDescriptor; N], 
}

const LOG_512_MIB : usize = 29;
const LOG_64_KIB : usize = 16;

const TRANSLATION_TABLES_LEVEL_2 : usize = physical::address_space_size() >> 29;  

static mut TRANSLATION_TABLES: TopLevelTables<TRANSLATION_TABLES_LEVEL_2> = TopLevelTables {
    level_3: [[PageDescriptor(0); 8192]; TRANSLATION_TABLES_LEVEL_2],
    level_2: [TableDescriptor(0); TRANSLATION_TABLES_LEVEL_2],    
};

pub unsafe fn setup_transaltion_tables(kernel_range: (usize, usize), device_range: (usize, usize)) {
    
    let mut tables = &mut TRANSLATION_TABLES;
    for (l2_nr, l2_entry) in tables.level_2.iter_mut().enumerate() {
        *l2_entry = tables.level_3[l2_nr].base_addr().into();

        for (l3_nr, l3_entry) in tables.level_3[l2_nr].iter_mut().enumerate() {
            let virt_addr = (l2_nr << LOG_512_MIB) + (l3_nr << LOG_64_KIB);

            let (output_addr, attribute_fields) =
                bsp::virt_mem_layout().get_virt_addr_properties(virt_addr)?;

            *l3_entry = PageDescriptor::new(output_addr, attribute_fields);
        }
    }

}
