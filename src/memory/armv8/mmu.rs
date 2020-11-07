use cortex_a::regs::*;

use super::translation_tables::*;
use crate::memory::memory_controler::KERNEL_RW_;
use crate::memory::memory_controler::{AttributeFields, Granule, Translation, MEMORY_LAYOUT};
struct MMU {}
const MEMORY_SIZE_BITS: usize = 36;
const MEMORY_REGION_OFFSET: usize = 64 - MEMORY_SIZE_BITS;
const MEMORY_SIZE_GIB: usize = 1 << (MEMORY_SIZE_BITS + 1 - 30); // 128 GIB

#[repr(C, align(4096))]
struct Level1MemoryTable {
    table_1g: [TableRecord; MEMORY_SIZE_GIB], // EACH 1GB
                                              //     reserved: [u64; 512 - MEMORY_SIZE_GIB],
                                              //     tables_2m: [[TableRecord; 512]; MEMORY_SIZE_GIB],      // EACH 2MB
                                              //     pages_4k: [[[PageRecord; 512]; 512]; MEMORY_SIZE_GIB], // EACH 4KB
}

impl Level1MemoryTable {
    fn fill(&mut self) {
        crate::println!("{}", MEMORY_SIZE_GIB);

        for memory_range in &MEMORY_LAYOUT {
            let step = match &memory_range.granule {
                Granule::Page4KiB => 1 << 12,
                Granule::Block2MiB => 1 << 21,
                Granule::Block1GiB => 1 << 30,
            };
            let offset = if let Translation::Offset(value) = memory_range.translation {
                value - (memory_range.virtual_range)().start
            } else {
                0
            };
            for address in (memory_range.virtual_range)().step_by(step) {
                self.map_memory(
                    address,
                    offset,
                    &memory_range.attribute_fields,
                    memory_range.granule,
                );
            }
        }
    }
    fn map_memory(
        &mut self,
        address: usize,
        offset: usize,
        memory_attributes: &AttributeFields,
        granule: Granule,
    ) {
        let table_layout =
            alloc::alloc::Layout::from_size_align(4096, 4096).expect("Couldnt crate layout");

        let level_1 = address >> 30;
        if let Granule::Block1GiB = granule {
            self.table_1g[level_1] =
                PageRecord::new(address + offset, *memory_attributes, true).into();
            return;
        }
        if !self.table_1g[level_1].is_valid() {
            self.table_1g[level_1] =
                (unsafe { alloc::alloc::alloc_zeroed(table_layout) } as usize).into();
        }
        let level_2 = (address - (level_1 << 30)) >> 21;
        let table_2m = unsafe { &mut *self.table_1g[level_1].next_table() };

        if let Granule::Block2MiB = granule {
            table_2m[level_2] = PageRecord::new(address + offset, *memory_attributes, true).into();
            return;
        }

        if !table_2m[level_2].is_valid() {
            table_2m[level_2] =
                (unsafe { alloc::alloc::alloc_zeroed(table_layout) } as usize).into();
        }
        let level_3 = (address - (level_1 << 30) - (level_2 << 21)) >> 12;
        let table_3k = unsafe { &mut *table_2m[level_2].next_page() };

        table_3k[level_3] = PageRecord::new(address + offset, *memory_attributes, false);
    }
}

#[cfg(not(feature = "raspi3"))]
pub const MEMORY_SIZE: usize = 4;
#[cfg(feature = "raspi3")]
pub const MEMORY_SIZE: usize = 1;

static mut MEMORY_TABLE: *mut Level1MemoryTable = core::ptr::null_mut();

///
/// # Safety
/// It is caller responsibility to ensure that only one reference to Level1MemoryTable lives.
unsafe fn get_base_memory_table() -> &'static mut Level1MemoryTable {
    if MEMORY_TABLE.is_null() {
        use alloc::boxed::Box;
        let mut boxed_table: Box<Level1MemoryTable> = Box::new_zeroed().assume_init();
        boxed_table.fill();
        MEMORY_TABLE = Box::leak(boxed_table) as *mut _;
    }
    &mut *MEMORY_TABLE
}
///
/// # Safety
/// Should be only called once before MMU is Initialized

pub unsafe fn add_translation(p_address: usize, v_address: usize) {
        (&mut *MEMORY_TABLE).map_memory(
            v_address,
            p_address - v_address,
            &KERNEL_RW_,
            Granule::Page4KiB,
        )
}

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
