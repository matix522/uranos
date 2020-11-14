use register::{mmio::*, register_bitfields};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageRecord(pub u64);
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct TableRecord(pub u64);

impl From<PageRecord> for TableRecord {
    fn from(val: PageRecord) -> Self {
        TableRecord(val.0)
    }
}

// A level 1 table descriptor, as per ARMv8-A Architecture Reference Manual Figure D4-15.
register_bitfields! {u64,
    STAGE1_TABLE_1_DESCRIPTOR [
        /// Physical address of the next page table.
        NEXT_LEVEL_TABLE_ADDR_4KiB OFFSET(12) NUMBITS(36) [], // [47:12]

        TYPE  OFFSET(1) NUMBITS(1) [
            OldBlock = 0,
            Table = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

// A level 3 page descriptor, as per ARMv8-A Architecture Reference Manual Figure D4-17.
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        /// Execute-never.
        XN       OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next page table (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR_4KiB OFFSET(12) NUMBITS(36) [], // [47:12]

        /// Access flag.
        AF       OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Shareability field.
        SH       OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11
        ],

        /// Access Permissions.
        AP       OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11
        ],

        /// Memory attributes index into the MAIR_EL1 register.
        AttrIndx OFFSET(2) NUMBITS(3) [],

        TYPE     OFFSET(1) NUMBITS(1) [
            OldBlock = 0,
            Page = 1
        ],

        VALID    OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}
pub const FOUR_KIB_SHIFT: usize = 12;
pub const TWO_MIB_SHIFT: usize = 21;
pub const ONE_GIB_SHIFT: usize = 30;

pub(super) trait BaseAddr<U> {
    fn as_addr(&self) -> U;
}

impl<T, const N: usize> BaseAddr<usize> for [T; N] {
    fn as_addr(&self) -> usize {
        self as *const T as usize
    }
}

impl PageRecord {
    pub fn new(output_addr: usize, attribute_fields: AttributeFields, is_block: bool) -> Self {
        let shifted = output_addr >> FOUR_KIB_SHIFT;
        let val = (STAGE1_PAGE_DESCRIPTOR::VALID::True
            + STAGE1_PAGE_DESCRIPTOR::AF::True
            + attribute_fields.into()
            + if is_block {
                STAGE1_PAGE_DESCRIPTOR::TYPE::OldBlock
            } else {
                STAGE1_PAGE_DESCRIPTOR::TYPE::Page
            }
            + STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_4KiB.val(shifted as u64))
        .value;

        Self(val)
    }
}
impl core::fmt::Display for PageRecord {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val: ReadWrite<u64, STAGE1_PAGE_DESCRIPTOR::Register> =
            unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        val.set(self.0);
        if val.matches_all(STAGE1_PAGE_DESCRIPTOR::PXN::False) {
            write!(f, "Executable ")?;
        } else {
            write!(f, "Non-Executable ")?;
        }
        if val.matches_all(STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable) {
            write!(f, "Outer Shareable ")?;
        } else if val.matches_all(STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable) {
            write!(f, "Inner Shareable ")?;
        } else {
            write!(f, "Unknown Shareabilty ")?;
        }
        if val.matches_any(STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1) {
            write!(f, "Read Only ")?;
        } else {
            write!(f, "Read Write ")?;
        }
        if val.read(STAGE1_PAGE_DESCRIPTOR::AttrIndx) == 0 {
            write!(f, "Device ")?;
        } else {
            write!(f, "DRAM ")?;
        }
        writeln!(
            f,
            "Of address {:#018x}",
            val.read(STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_4KiB) * 4096
        )?;

        Ok(())
    }
}

impl core::convert::From<usize> for TableRecord {
    fn from(next_lvl_table_addr: usize) -> Self {
        let shifted = next_lvl_table_addr >> FOUR_KIB_SHIFT;
        let value = (STAGE1_TABLE_1_DESCRIPTOR::VALID::True
            + STAGE1_TABLE_1_DESCRIPTOR::TYPE::Table
            + STAGE1_TABLE_1_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_4KiB.val(shifted as u64))
        .value;
        TableRecord(value)
    }
}
use crate::memory::armv8::mmu;
use crate::memory::memory_controler::*;
/// Convert the kernel's generic memory range attributes to HW-specific attributes of the MMU.
impl core::convert::From<AttributeFields>
    for register::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(attribute_fields: AttributeFields) -> Self {
        // Memory attributes.
        let mut desc = match attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mmu::Mair::NormalCachableDRAM as u64)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mmu::Mair::Device as u64)
            }
        };

        // Access Permissions.
        desc += match attribute_fields.acc_perms {
            AccessPermissions::KernelReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            AccessPermissions::KernelReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
            AccessPermissions::UserReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1_EL0,
            AccessPermissions::UserReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1_EL0,
        };

        // Execute Never.
        desc += if attribute_fields.executable {
            STAGE1_PAGE_DESCRIPTOR::PXN::False + STAGE1_PAGE_DESCRIPTOR::XN::False
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::True + STAGE1_PAGE_DESCRIPTOR::XN::True
        };

        desc
    }
}

pub enum TableEntryType {
    Invalid = 0b00,
    Block = 0b01,
    TableOrPage = 0b11,
}

impl TableRecord {
    pub fn is_valid(&self) -> bool {
        (self.0 & 0b1) == 1
    }
    pub fn get_type(&self) -> TableEntryType {
        match self.0 & 0b11 {
            0b01 => TableEntryType::Block,
            0b11 => TableEntryType::TableOrPage,
            _ => TableEntryType::Invalid,
        }
    }
    pub fn get_address(&self) -> usize {
        const MASK: u64 = 0xffff_ffff_f000;
        (self.0 & MASK) as usize
    }
    /// # Safety
    /// self must be correct entry of 1,2 level describing a table of tables
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn next_table(&self) -> &mut [TableRecord; 512] {
        &mut *((self.0 & (((1 << 36) - 1) << 12)) as *mut [TableRecord; 512])
    }
    /// # Safety
    /// self must be correct entry of 1,2 level describing a table of page records
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn next_page(&self) -> &mut [PageRecord; 512] {
        &mut *((self.0 & (((1 << 36) - 1) << 12)) as *mut [PageRecord; 512])
    }
}

impl PageRecord {
    pub fn get_type(&self) -> TableEntryType {
        match self.0 & 0b11 {
            0b01 => TableEntryType::Block,
            0b11 => TableEntryType::TableOrPage,
            _ => TableEntryType::Invalid,
        }
    }
    pub fn get_address(&self) -> usize {
        const MASK: u64 = 0xffff_ffff_f000;
        (self.0 & MASK) as usize
    }
}
