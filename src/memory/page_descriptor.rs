use register::register_bitfields;
use core::convert;
use super::attributes::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(super) struct PageDescriptor(pub u64);

/// Constants for indexing the MAIR_EL1.
#[allow(dead_code)]
mod mair {
    pub const DEVICE: u64 = 0;
    pub const NORMAL: u64 = 1;
}

// A level 3 page descriptor, as per AArch64 Reference Manual Figure D4-17.
register_bitfields! {u64,
    STAGE1_PAGE_DESCRIPTOR [
        /// Privileged execute-never.
        PXN      OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1
        ],

        /// Physical address of the next page table (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

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

/// Convert the kernel's generic memory range attributes to HW-specific attributes of the MMU.
impl convert::From<AttributeFields>
    for register::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(attribute_fields: AttributeFields) -> Self {
        // Memory attributes.
        let mut desc = match attribute_fields.mem_attributes {
            MemAttributes::CacheableDRAM => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mair::NORMAL)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(mair::DEVICE)
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

impl PageDescriptor {
    fn new(output_addr: usize, attribute_fields: AttributeFields) -> PageDescriptor {
        let shifted = output_addr >> super::LOG_64_KIB;
        let val = (STAGE1_PAGE_DESCRIPTOR::VALID::True
            + STAGE1_PAGE_DESCRIPTOR::AF::True
            + attribute_fields.into()
            + STAGE1_PAGE_DESCRIPTOR::TYPE::Table
            + STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted as u64))
        .value;

        PageDescriptor(val)
    }
}