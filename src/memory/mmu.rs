use cortex_a::regs::MAIR_EL1;
use register::register_bitfields;
use register::cpu::RegisterReadWrite;

register_bitfields! {u64,
    STAGE1_TABLE_DESCRIPTOR [
        /// Physical address of the next page table.
        Next_Level_Table_Address_64KiB OFFSET(16) NUMBITS(32) [], // [47:16]

        Type  OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1
        ],

        Valid OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct TableDescriptor(u64);

pub enum MMUError {}

static mut LEVEL_0_TABLE : [TableDescriptor; 512] = [TableDescriptor(0);512];
static mut LEVEL_1_TABLE : [TableDescriptor; 512] = [TableDescriptor(0);512];
static mut LEVEL_2_TABLE : [TableDescriptor; 512] = [TableDescriptor(0);512];
static mut LEVEL_3_TABLE : [PageDescriptor; 512] = [PageDescriptor(0);512];



fn setup_mair() {
    MAIR_EL1.write(
        MAIR_EL1::Attr1_HIGH::Memory_OuterWriteBack_NonTransient_ReadAlloc_WriteAlloc
            + MAIR_EL1::Attr1_LOW_MEMORY::InnerWriteBack_NonTransient_ReadAlloc_WriteAlloc
            + MAIR_EL1::Attr0_HIGH::Device
            + MAIR_EL1::Attr0_LOW_DEVICE::Device_nGnRE
    );

}

// unsafe fn create_entries() -> Result<(), &'static str> {
//     for (l2_nr, l2_entry) in TABLES.lvl2.iter_mut().enumerate() {
//         *l2_entry = TABLES.lvl3[l2_nr].base_addr_usize().into();

//         for (l3_nr, l3_entry) in TABLES.lvl3[l2_nr].iter_mut().enumerate() {
//             let virt_addr = (l2_nr << FIVETWELVE_MIB_SHIFT) + (l3_nr << SIXTYFOUR_KIB_SHIFT);

//             let (output_addr, attribute_fields) =
//                 bsp::virt_mem_layout().get_virt_addr_properties(virt_addr)?;

//             *l3_entry = PageDescriptor::new(output_addr, attribute_fields);
//         }
//     }

//     Ok(())
// }


pub struct MMU;

impl MMU {
    unsafe fn init() -> Result<(), MMUError> {
        Ok(())
    }
}
