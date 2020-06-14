use register::{mmio::*, register_bitfields};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageRecord(pub u64);
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Table2Record(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Table1Record(pub u64);
// A level 1 table descriptor, as per ARMv8-A Architecture Reference Manual Figure D4-15.
register_bitfields! {u64,
    STAGE1_TABLE_1_DESCRIPTOR [
        /// Physical address of the next page table.
        NEXT_LEVEL_TABLE_ADDR_4KiB OFFSET(30) NUMBITS(18) [], // [47:30]

        TYPE  OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

// A level 2 table descriptor, as per ARMv8-A Architecture Reference Manual Figure D4-15.
register_bitfields! {u64,
    STAGE1_TABLE_2_DESCRIPTOR [
        /// Physical address of the next page table.
        NEXT_LEVEL_TABLE_ADDR_4KiB OFFSET(21) NUMBITS(27) [], // [47:21]

        TYPE  OFFSET(1) NUMBITS(1) [
            Block = 0,
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
        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next page table (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR_4KiB OFFSET(12) NUMBITS(36) [], // [47:16]

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
            Block = 0,
            Table = 1
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
    pub fn new(output_addr: usize, attribute_fields: AttributeFields) -> Self {
        let shifted = output_addr >> FOUR_KIB_SHIFT;
        let val = (STAGE1_PAGE_DESCRIPTOR::VALID::True
            + STAGE1_PAGE_DESCRIPTOR::AF::True
            + attribute_fields.into()
            + STAGE1_PAGE_DESCRIPTOR::TYPE::Block
            + STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_4KiB.val(shifted as u64))
        .value;

        Self(val)
    }
}
impl core::fmt::Display for PageRecord {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val: ReadWrite<u64, STAGE1_PAGE_DESCRIPTOR::Register> =
            unsafe { core::mem::uninitialized() };
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

        //     /// Physical address of the next page table (lvl2) or the page descriptor (lvl3).
        //     OUTPUT_ADDR_4KiB OFFSET(12) NUMBITS(36) [], // [47:16]

        //     /// Access Permissions.
        //     AP       OFFSET(6) NUMBITS(2) [
        //         RW_EL1 = 0b00,
        //         RW_EL1_EL0 = 0b01,
        //         RO_EL1 = 0b10,
        //         RO_EL1_EL0 = 0b11
        //     ],

        //     /// Memory attributes index into the MAIR_EL1 register.
        //     AttrIndx OFFSET(2) NUMBITS(3) [],

        //     TYPE     OFFSET(1) NUMBITS(1) [
        //         Block = 0,
        //         Table = 1
        //     ],

        //     VALID    OFFSET(0) NUMBITS(1) [
        //         False = 0,
        //         True = 1
        //     ]

        Ok(())
    }
}

impl core::convert::From<usize> for Table1Record {
    fn from(next_lvl_table_addr: usize) -> Self {
        let shifted = next_lvl_table_addr >> FOUR_KIB_SHIFT;
        let value = (STAGE1_TABLE_1_DESCRIPTOR::VALID::True
            + STAGE1_TABLE_1_DESCRIPTOR::TYPE::Table
            + STAGE1_TABLE_1_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_4KiB.val(shifted as u64))
        .value;
        Table1Record(value)
    }
}
impl core::convert::From<usize> for Table2Record {
    fn from(next_lvl_table_addr: usize) -> Self {
        let shifted = next_lvl_table_addr >> FOUR_KIB_SHIFT;
        let value = (STAGE1_TABLE_2_DESCRIPTOR::VALID::True
            + STAGE1_TABLE_2_DESCRIPTOR::TYPE::Table
            + STAGE1_TABLE_2_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_4KiB.val(shifted as u64))
        .value;
        Table2Record(value)
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
            AccessPermissions::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // Execute Never.
        desc += if attribute_fields.execute_never {
            STAGE1_PAGE_DESCRIPTOR::PXN::True
        } else {
            STAGE1_PAGE_DESCRIPTOR::PXN::False
        };

        desc
    }
}
