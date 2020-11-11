use cortex_a::regs::*;

use core::ptr::null_mut;

use super::translation_tables::*;
use crate::memory::memory_controler::{
    AddressSpace, AttributeFields, Granule, Translation, PHYSICAL_MEMORY_LAYOUT,
};
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
    fn fill(&mut self) -> Result<(), TranslationError> {
        crate::println!("{}", MEMORY_SIZE_GIB);

        for memory_range in &PHYSICAL_MEMORY_LAYOUT {
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
                )?;
            }
        }
        Ok(())
    }
    fn map_memory(
        &mut self,
        address: usize,
        offset: usize,
        memory_attributes: &AttributeFields,
        granule: Granule,
    ) -> Result<(), TranslationError> {
        let table_layout =
            alloc::alloc::Layout::from_size_align(4096, 4096).expect("Could not create layout.");
        let level_1 = address >> 30;
        if crate::config::debug_mmu() {
            crate::println!("L1 {:x}", level_1);
        }
        if let Granule::Block1GiB = granule {
            self.table_1g[level_1] =
                PageRecord::new(address + offset, *memory_attributes, true).into();
            return Ok(());
        }
        if !self.table_1g[level_1].is_valid() {
            self.table_1g[level_1] =
                (unsafe { alloc::alloc::alloc_zeroed(table_layout) } as usize).into();
        }
        let level_2 = (address - (level_1 << 30)) >> 21;
        if crate::config::debug_mmu() {
            crate::println!("L1 {:x}, L2 {:x}", level_1, level_2);
        }
        let table_2m = unsafe { &mut *self.table_1g[level_1].next_table() };

        if let Granule::Block2MiB = granule {
            table_2m[level_2] = PageRecord::new(address + offset, *memory_attributes, true).into();
            return Ok(());
        }

        if !table_2m[level_2].is_valid() {
            table_2m[level_2] =
                (unsafe { alloc::alloc::alloc_zeroed(table_layout) } as usize).into();
        }

        let level_3 = (address - (level_1 << 30) - (level_2 << 21)) >> 12;
        if crate::config::debug_mmu() {
            crate::println!("L1 {:x}, L2 {:x}, L3 {:x}", level_1, level_2, level_3);
        }
        if crate::config::debug_mmu() {
            crate::println!(
                "L1 {:x}, L2 {:x}, L3 {:x}, ADDRESS {:x} ",
                level_1,
                level_2,
                level_3,
                address + offset
            );
        }
        let table_3k = unsafe { &mut *table_2m[level_2].next_page() };

        table_3k[level_3] = PageRecord::new(address + offset, *memory_attributes, false);
        Ok(())
    }
}
#[derive(Debug)]
pub enum TranslationError {}

static MEMORY_CONTROLER: MMU = MMU::new();

static mut BASE_USER_MEMORY_TABLE: *mut Level1MemoryTable = null_mut();
static mut BASE_KERNEL_MEMORY_TABLE: *mut Level1MemoryTable = null_mut();

use crate::memory::memory_controler::RangeDescriptor;

pub unsafe fn map_memory(
    address_space: AddressSpace,
    memory_range: &RangeDescriptor,
) -> Result<(), TranslationError> {
    let translation = if let AddressSpace::Kernel = address_space {
        &mut *BASE_KERNEL_MEMORY_TABLE
    } else {
        &mut *BASE_USER_MEMORY_TABLE
    };
    let step = match &memory_range.granule {
        Granule::Page4KiB => 1 << 12,
        Granule::Block2MiB => 1 << 21,
        Granule::Block1GiB => 1 << 30,
    };
    let offset = if let Translation::Offset(value) = memory_range.translation {
        value - memory_range.virtual_range.start
    } else {
        0
    };
    crate::println!("OFFSET {:x}", offset);

    let range = memory_range.virtual_range.clone();
    for address in range.step_by(step) {
        translation.map_memory(
            address,
            offset,
            &memory_range.attribute_fields,
            memory_range.granule,
        )?;
    }
    Ok(())
}

pub unsafe fn unmap_memory(address_space: AddressSpace, memory_range: RangeDescriptor) {}

/// # Safety
/// MMU needs to be turned off, before this function is called.
pub unsafe fn init_mmu() -> Result<(), &'static str> {
    use cortex_a::barrier;

    // Fail early if translation granule is not supported.
    if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran4::Supported) {
        return Err("4 KiB translation granule not supported");
    }

    // Prepare the memory attribute indirection register.
    MEMORY_CONTROLER.setup_mair();
    {
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::new::<Level1MemoryTable>();

        let alloc_table = || alloc_zeroed(layout) as *mut Level1MemoryTable;

        BASE_USER_MEMORY_TABLE = alloc_table();
        BASE_KERNEL_MEMORY_TABLE = alloc_table();
    }

    if BASE_KERNEL_MEMORY_TABLE.is_null() || BASE_USER_MEMORY_TABLE.is_null() {
        panic!("Memory map not allocated.");
    }
    let kernel_table = &mut *BASE_KERNEL_MEMORY_TABLE;
    let user_table = &mut *BASE_USER_MEMORY_TABLE;

    // Fill the table with initial configuration.
    kernel_table.fill();

    // Copy top level of initial configuration to user table.
    user_table.table_1g = kernel_table.table_1g;

    let kernel_address = kernel_table.table_1g.as_ptr() as u64;
    let user_address = user_table.table_1g.as_ptr() as u64;

    crate::println!("MMU BASE KERNEL TABLE: {:#018x}", kernel_address);
    crate::println!("MMU BASE USER TABLE:   {:#018x}", user_address);

    TTBR1_EL1.set_baddr(kernel_address);
    TTBR0_EL1.set_baddr(user_address);

    MEMORY_CONTROLER.configure_translation_control();

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
    const fn new() -> Self {
        MMU {}
    }

    unsafe fn setup_mair(&self) {
        MAIR_EL1.write(
            // Attribute 1 - Cacheable normal DRAM.
            MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc +
            MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc +
            // Attribute 0 - Device. 
            MAIR_EL1::Attr0_Normal_Outer::Device +
            MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
        );
    }
    unsafe fn configure_translation_control(&self) {
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
