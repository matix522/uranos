use core::ops::RangeBounds;
use cortex_a::regs::*;

use super::translation_tables::*;

struct MMU {
    // main_table: TopLevelTables<N>,
}
const MEMORY_SIZE_BITS : usize = 36;
const MEMORY_REGION_OFFSET : usize = 64 - MEMORY_SIZE_BITS;
const MEMORY_SIZE_GIB : usize = 1 << (MEMORY_SIZE_BITS - 30); // 64 GIB

#[repr(C, align(4096))]
struct BaseMemoryTable {
    table_1g: [TableRecord; MEMORY_SIZE_GIB],            // EACH 1GB
    reserved: [u64; 512 - MEMORY_SIZE_GIB],
    tables_2m: [[TableRecord; 512]; MEMORY_SIZE_GIB],      // EACH 2MB
    pages_4k: [[[PageRecord; 512]; 512]; MEMORY_SIZE_GIB], // EACH 4KB
}

impl BaseMemoryTable {
    fn fill(&mut self) {        
        'outer: for (n, block_1g) in self.pages_4k.iter_mut().enumerate() {
            crate::print!("{} GIB ", n);
            for (i, block_2m) in block_1g.iter_mut().enumerate() {
                for (j, page_4k) in block_2m.iter_mut().enumerate() {
                    use super::super::memory_controler::*;                    
                    
                    let addr = (1 << 30) * n + (1 << 21) * i + (1 << 12) * j;
                    if let Some(range) = MEMORY_LAYOUT.iter().filter(|range| (range.virtual_range)().contains(&addr)).next() {
                        *page_4k =  PageRecord::new(addr, range.attribute_fields, false)
                    }
                } // 0xfffffff00008b010
            }
            crate::println!("Done");

        }
        crate::println!("value");
        for (n, table_1g) in self.tables_2m.iter_mut().enumerate() {
            for (i, table_2m) in table_1g.iter_mut().enumerate() {
                *table_2m = self.pages_4k[n][i].as_addr().into();
            }
        }
        // for (n, table_1g) in self.tables_2m_a.iter_mut().enumerate() {
        //     for (i, table_2m) in table_1g.iter_mut().enumerate() {
        //         *table_2m = self.pages_4k[n][i].as_addr().into();
        //     }
        // }
        for n in 0..MEMORY_SIZE_GIB {
            self.table_1g[n] = self.tables_2m[n].as_addr().into();
        }
    }
}

#[cfg(not(feature = "raspi3"))]
pub const MEMORY_SIZE: usize = 4;
#[cfg(feature = "raspi3")]
pub const MEMORY_SIZE: usize = 1;

static mut MEMORY_TABLE: *mut BaseMemoryTable = core::ptr::null_mut();

///
/// # Safety
/// It is caller responsibility to ensure that only one reference to BaseMemoryTable lives.
unsafe fn get_base_memory_table() -> &'static mut BaseMemoryTable {
    if MEMORY_TABLE.is_null() {
        use alloc::boxed::Box;
        let mut boxed_table: Box<BaseMemoryTable> = Box::new_zeroed().assume_init();
        boxed_table.fill();
        MEMORY_TABLE = Box::leak(boxed_table) as *mut _;
    }
    &mut *MEMORY_TABLE
}
///
/// # Safety
/// Should be only called once before MMU is Initialized
pub unsafe fn test() -> Result<(), &'static str> {
    use cortex_a::barrier;
    let mut m = MMU::new();

    // Fail early if translation granule is not supported. Both RPis support it, though.
    if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran4::Supported) {
        return Err("4 KiB translation granule not supported");
    }

    // Prepare the memory attribute indirection register.
    m.setup_mair();

    // Populate page tables.
    m.populate_tables();

    let translation = get_base_memory_table();

    let addr = translation.table_1g.as_ptr() as u64;

    crate::println!("MMU BASE TABLE: {:#018x}", addr);

    TTBR0_EL1.set_baddr(addr);
    TTBR1_EL1.set_baddr(addr);

    m.configure_translation_control();

    crate::println!("Start MMU");

    // Switch the MMU on.
    //
    // First, force all previous changes to be seen before the MMU is enabled.
    barrier::isb(barrier::SY);

    // Enable the MMU and turn on data and instruction caching.
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    // Force MMU init to complete before next instruction.
    barrier::isb(barrier::SY);

    crate::println!("MMU Enabled");
    Ok(())
}

impl MMU {
    fn new() -> Self {
        MMU {}
    }

    unsafe fn setup_mair(&mut self) {
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM.
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
            MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +
            // Attribute 0 - Device. 
            MAIR_EL1::Attr0_Normal_Outer::Device +
            MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }
    unsafe fn populate_tables(&mut self) {}
    unsafe fn configure_translation_control(&mut self) {
        let ips = ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::PARange);

        TCR_EL1.write(
            TCR_EL1::TBI0::Ignored
                + TCR_EL1::IPS.val(ips)

                + TCR_EL1::TG0.val(0b00)//::KiB_4
                + TCR_EL1::SH0::Inner
                + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD0::EnableTTBR0Walks
                + TCR_EL1::T0SZ.val(MEMORY_REGION_OFFSET as u64) // TTBR0 spans 64 GiB total.

                + TCR_EL1::TG1.val(0b10)//::KiB_4
                + TCR_EL1::SH1::Inner
                + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
                + TCR_EL1::EPD1::EnableTTBR1Walks
                + TCR_EL1::T1SZ.val(MEMORY_REGION_OFFSET as u64), // TTBR1 spans 64 GiB total.
        );
    }
}

#[repr(u64)]
pub enum Mair {
    Device = 0,
    NormalCachableDRAM = 1,
}
