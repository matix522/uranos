use core::convert;
use register::register_bitfields;

// A table descriptor, as per AArch64 Reference Manual Figure D4-15.
register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        /// Physical address of the next page table.
        NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

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

impl convert::From<u64> for TableDescriptor {
    fn from(next_lvl_table_addr: u64) -> Self {
        let shifted = next_lvl_table_addr >> super::LOG_64_KIB;
        let val = (STAGE1_TABLE_DESCRIPTOR::VALID::True
            + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
            + STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64))
        .value;

        TableDescriptor(val)
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(super) struct TableDescriptor(pub u64);
