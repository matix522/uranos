use cortex_a::regs::*;

use core::ptr::null_mut;

use super::translation_tables::*;
use crate::alloc::alloc::{alloc_zeroed, Layout};
use crate::memory::memory_controler::{
    AddressSpace, AttributeFields, Granule, Translation, PHYSICAL_MEMORY_LAYOUT,
};

struct MMU {}
const MEMORY_SIZE_BITS: usize = 36;
const MEMORY_REGION_OFFSET: usize = 64 - MEMORY_SIZE_BITS;
const MEMORY_SIZE_GIB: usize = 1 << (MEMORY_SIZE_BITS + 1 - 30); // 128 GIB

// enum WalkResult {
//     Level1(&mut TableRecord)
//     Level2(&mut TableRecord)
//     Level3(&mut TableRecord)
//     Invalid
// }

#[repr(C, align(4096))]
struct Level1MemoryTable {
    table_1g: [TableRecord; MEMORY_SIZE_GIB], // EACH 1GB
                                              //     reserved: [u64; 512 - MEMORY_SIZE_GIB],
                                              //     tables_2m: [[TableRecord; 512]; MEMORY_SIZE_GIB],      // EACH 2MB
                                              //     pages_4k: [[[PageRecord; 512]; 512]; MEMORY_SIZE_GIB], // EACH 4KB
}

impl Level1MemoryTable {
    unsafe fn fill(&mut self) -> Result<(), TranslationError> {
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

    unsafe fn translate(&mut self, address: usize) -> Result<usize, u64> {
        let level_1 = address >> 30;
        let level_2 = (address - (level_1 << 30)) >> 21;
        let level_3 = (address - (level_1 << 30) - (level_2 << 21)) >> 12;

        let last_bits = address & 0xfff;

        let level_1_entry = &mut self.table_1g[level_1];

        let table_2m = match level_1_entry.get_type() {
            TableEntryType::Invalid => return Err(level_1_entry.0),
            TableEntryType::Block => return Ok(level_1_entry.get_address() + last_bits),
            TableEntryType::TableOrPage => &mut *level_1_entry.next_table(),
        };
        let level_2_entry = &mut table_2m[level_2];

        let table_4k = match level_2_entry.get_type() {
            TableEntryType::Invalid => return Err(level_2_entry.0),
            TableEntryType::Block => return Ok(level_2_entry.get_address() + last_bits),
            TableEntryType::TableOrPage => &mut *level_2_entry.next_page(),
        };
        let level_3_entry = &mut table_4k[level_3];
        match level_3_entry.get_type() {
            TableEntryType::Invalid | TableEntryType::Block => Err(level_3_entry.0),
            TableEntryType::TableOrPage => Ok(level_3_entry.get_address() + last_bits),
        }
    }
    unsafe fn map_memory(
        &mut self,
        address: usize,
        offset: usize,
        memory_attributes: &AttributeFields,
        granule: Granule,
    ) -> Result<(), TranslationError> {
        let table_layout =
            Layout::from_size_align(4096, 4096).map_err(|_| TranslationError::IncorrectLayout)?;

        let level_1 = address >> 30;
        let level_2 = (address - (level_1 << 30)) >> 21;
        let level_3 = (address - (level_1 << 30) - (level_2 << 21)) >> 12;
        if crate::config::debug_mmu() {
            crate::println!(
                "L1: {}, L2: {}, L3: {}, ADDR: {:x}, PADDR: {:x}, OFF: {:x}",
                level_1,
                level_2,
                level_3,
                address,
                address + offset,
                offset
            );
        }
        let level_1_entry = &mut self.table_1g[level_1];

        let table_2m = match level_1_entry.get_type() {
            TableEntryType::Invalid if granule == Granule::Block1GiB => {
                *level_1_entry = PageRecord::new(address + offset, *memory_attributes, true).into();
                return Ok(());
            }
            TableEntryType::Invalid => {
                *level_1_entry = (alloc_zeroed(table_layout) as usize).into();
                Ok(level_1_entry.next_table())
            }
            TableEntryType::Block => Err(TranslationError::MappedHugePage),
            TableEntryType::TableOrPage if granule == Granule::Block1GiB => {
                Err(TranslationError::MappedTableLevel1)
            }
            TableEntryType::TableOrPage => Ok(&mut *level_1_entry.next_table()),
        };

        let level_2_entry = &mut table_2m?[level_2];
        let table_4k = match level_2_entry.get_type() {
            TableEntryType::Invalid if granule == Granule::Block2MiB => {
                *level_2_entry = PageRecord::new(address + offset, *memory_attributes, true).into();
                return Ok(());
            }
            TableEntryType::Invalid => {
                *level_2_entry = (alloc_zeroed(table_layout) as usize).into();
                Ok(level_2_entry.next_page())
            }
            TableEntryType::Block => Err(TranslationError::MappedLargePage),

            TableEntryType::TableOrPage if granule == Granule::Block2MiB => {
                Err(TranslationError::MappedTableLevel2)
            }
            TableEntryType::TableOrPage => Ok(&mut *level_2_entry.next_page()),
        };

        let level_3_entry = &mut table_4k?[level_3];
        match level_3_entry.get_type() {
            TableEntryType::Invalid | TableEntryType::Block => {
                *level_3_entry = PageRecord::new(address + offset, *memory_attributes, false);
                Ok(())
            }
            TableEntryType::TableOrPage => Err(TranslationError::MappedPage),
        }
    }

    unsafe fn unmap_memory(
        &mut self,
        address: usize,
        granule: Granule,
    ) -> Result<(), TranslationError> {
        // TODO: Fix memory leak
        let level_1 = address >> 30;
        let level_2 = (address - (level_1 << 30)) >> 21;
        let level_3 = (address - (level_1 << 30) - (level_2 << 21)) >> 12;

        let level_1_entry = &mut self.table_1g[level_1];

        let table_2m = match level_1_entry.get_type() {
            TableEntryType::Invalid => Err(TranslationError::InvalidHugePage),
            TableEntryType::Block if granule == Granule::Block1GiB => {
                *level_1_entry = TableRecord(0);
                return Ok(());
            }
            TableEntryType::Block => Err(TranslationError::MappedHugePage),
            TableEntryType::TableOrPage if granule == Granule::Block1GiB => {
                Err(TranslationError::MappedTableLevel1)
            }
            TableEntryType::TableOrPage => Ok(level_1_entry.next_table()),
        };

        let level_2_entry = &mut table_2m?[level_2];
        let table_4k = match level_2_entry.get_type() {
            TableEntryType::Invalid => Err(TranslationError::InvalidLargePage),
            TableEntryType::Block if granule == Granule::Block2MiB => {
                *level_2_entry = TableRecord(0);
                return Ok(());
            }
            TableEntryType::Block => Err(TranslationError::MappedLargePage),
            TableEntryType::TableOrPage if granule == Granule::Block2MiB => {
                Err(TranslationError::MappedTableLevel2)
            }
            TableEntryType::TableOrPage => Ok(level_2_entry.next_page()),
        };

        let level_3_entry = &mut table_4k?[level_3];
        match level_3_entry.get_type() {
            TableEntryType::Invalid | TableEntryType::Block => Err(TranslationError::InvalidPage),
            TableEntryType::TableOrPage => {
                *level_3_entry = PageRecord(0);
                Ok(())
            }
        }
    }
}
#[derive(Debug)]
pub enum TranslationError {
    IncorrectLayout,
    HugePageFound,
    MappedHugePage,
    MappedLargePage,
    MappedPage,
    MappedTableLevel1,
    MappedTableLevel2,
    InvalidHugePage,
    InvalidLargePage,
    InvalidPage,
}

static MEMORY_CONTROLER: MMU = MMU::new();

static mut BASE_USER_MEMORY_TABLE: *mut Level1MemoryTable = null_mut();
static mut BASE_KERNEL_MEMORY_TABLE: *mut Level1MemoryTable = null_mut();

use crate::memory::memory_controler::RangeDescriptor;

pub unsafe fn translate(address_space: AddressSpace, address: usize) -> Result<usize, u64> {
    let translation = if let AddressSpace::Kernel = address_space {
        &mut *BASE_KERNEL_MEMORY_TABLE
    } else {
        &mut *BASE_USER_MEMORY_TABLE
    };
    translation.translate(address)
}

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

pub unsafe fn unmap_memory(
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

    let range = memory_range.virtual_range.clone();
    for address in range.step_by(step) {
        translation.unmap_memory(address, memory_range.granule)?;
    }
    Ok(())
}

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
    kernel_table.fill().unwrap();

    // Copy top level of initial configuration to user table.
    user_table.table_1g = kernel_table.table_1g;

    let kernel_address = kernel_table.table_1g.as_ptr() as u64;
    let user_address = user_table.table_1g.as_ptr() as u64;
    if crate::config::debug_mmu() {
    crate::println!("MMU BASE KERNEL TABLE: {:#018x}", kernel_address);
    crate::println!("MMU BASE USER TABLE:   {:#018x}", user_address);
    }
    TTBR1_EL1.set_baddr(kernel_address);
    TTBR0_EL1.set_baddr(user_address);

    MEMORY_CONTROLER.configure_translation_control();

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
