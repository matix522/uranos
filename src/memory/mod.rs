pub mod allocator;
pub mod attributes;
pub mod layout;
pub mod page_descriptor;
pub mod table_descriptor;

pub use attributes::*;
use cortex_a::regs::*;
pub use layout::*;
use page_descriptor::*;
use table_descriptor::*;

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

        // pub const GPIO_BASE:       usize = BASE + 0x0020_0000;
        // pub const UART_BASE:       usize = BASE + 0x0020_1000;
        #[cfg(feature = "raspi3")]
        pub const END:             usize =        0x4100_0000;
        #[cfg(not(feature = "raspi3"))]
        pub const END:             usize =        0xFFFF_FFFF;

    }
    pub const fn address_space_size() -> usize {
        MEMORY_END + 1
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

#[repr(C, align(65536))]
pub struct TopLevelTables<const N: usize> {
    level_3: [[PageDescriptor; 8192]; N],
    level_2: [TableDescriptor; N], // 512 MB each
}

const LOG_512_MIB: usize = 29;
const LOG_64_KIB: usize = 16;

const TRANSLATION_TABLES_LEVEL_2: usize = (physical::address_space_size() >> LOG_512_MIB) + 1;

static mut TRANSLATION_TABLES: TopLevelTables<TRANSLATION_TABLES_LEVEL_2> = TopLevelTables {
    level_3: [[PageDescriptor(0); 8192]; TRANSLATION_TABLES_LEVEL_2],
    level_2: [TableDescriptor(0); TRANSLATION_TABLES_LEVEL_2],
};
pub unsafe fn get_translation_table_address() -> u64 {
    TRANSLATION_TABLES.level_2.base_addr()
}
pub unsafe fn setup_mair() {
    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Normal_Outer::Device
            + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
    );
}

pub unsafe fn setup_transaltion_tables() -> Result<(), &'static str> {
    let tables = &mut TRANSLATION_TABLES;
    layout::LAYOUT.print_layout();
    for (l2_nr, l2_entry) in tables.level_2.iter_mut().enumerate() {
        *l2_entry = tables.level_3[l2_nr].base_addr().into();
        // crate::println!("{:#018x}", (*l2_entry).0);
        // crate::println!("{:#012x}", tables.level_3[l2_nr].base_addr());
        crate::println!("{}", l2_nr);
        for (l3_nr, l3_entry) in tables.level_3[l2_nr].iter_mut().enumerate() {
            let virt_addr = (l2_nr << LOG_512_MIB) + (l3_nr << LOG_64_KIB);
            // crate::println!("bbb");

            let (output_addr, attribute_fields) =
                layout::LAYOUT.get_virt_addr_properties(virt_addr)?;

            *l3_entry = PageDescriptor::new(output_addr, attribute_fields);
            // crate::println!("{:#018x}",(*l3_entry).0);
        }
    }
    Ok(())
}

/// Configure various settings of stage 1 of the EL1 translation regime.
pub unsafe fn configure_translation_control() {
    let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);
    TCR_EL1.write(
        TCR_EL1::TBI0::Ignored
            + TCR_EL1::IPS.val(ips)
            + TCR_EL1::TG0::KiB_64
            + TCR_EL1::SH0::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::T0SZ.val(32), // TTBR0 spans 4 GiB total.
    );
}
