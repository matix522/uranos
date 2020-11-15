use crate::memory::armv8::mmu::*;
use crate::memory::armv8::translation_tables::*;
use crate::memory::memory_controler::*;
use crate::utils::binary_info::BinaryInfo;
use alloc::alloc::{alloc_zeroed, Layout};
use alloc::boxed::Box;
use core::ops::Range;
use core::ptr::null_mut;
pub struct TaskMemoryManager {
    pub additional_table_hack: Box<Level1MemoryTable>,
    memory_descriptors: MemoryMap,
}

impl Default for TaskMemoryManager {
    fn default() -> Self {
        let mut memory_map = MemoryMap::new();
        let binary_info = BinaryInfo::get();

        const PROCESS_OFFSET: usize = 0x1_0000_0000;
        let make_virtual = |range: &Range<usize>| {
            ((PROCESS_OFFSET | range.start) & (!crate::KERNEL_OFFSET))
                ..((PROCESS_OFFSET | range.end) & (!crate::KERNEL_OFFSET))
        };
        let get_offset = |range: &Range<usize>| Translation::Offset(range.start & (!crate::KERNEL_OFFSET));

        memory_map.insert(
            "Static Task Data and Code".into(),
            RangeDescriptor {
                virtual_range: make_virtual(&binary_info.read_only),
                translation: get_offset(&binary_info.read_only),
                attribute_fields: USER_R_X,
                granule: Granule::Page4KiB,
            },
        );
        memory_map.insert(
            "Mutable Common Data".into(),
            RangeDescriptor {
                virtual_range: make_virtual(&binary_info.read_write),
                translation: get_offset(&binary_info.read_write),
                attribute_fields: USER_RW_,
                granule: Granule::Page4KiB,
            },
        );
        memory_map.insert(
            "Allocator Page".into(),
            RangeDescriptor {
                virtual_range: make_virtual(&binary_info.allocator),
                translation: get_offset(&binary_info.allocator),
                attribute_fields: USER_RW_,
                granule: Granule::Page4KiB,
            },
        );
        memory_map.insert(
            "Initial Common Heap".into(),
            RangeDescriptor {
                virtual_range: make_virtual(&binary_info.heap),
                translation: get_offset(&binary_info.heap),
                attribute_fields: USER_RW_,
                granule: Granule::Page4KiB,
            },
        );
        let task_local_pages = {
            binary_info.task_local.start
                ..binary_info.task_local.end
                    + (binary_info.task_local.end as *const u8).align_offset(4096)
        };
        crate::println!(
            "Alligned {:x} - {:x}",
            task_local_pages.start,
            task_local_pages.end
        );
        let page_address =
            unsafe { alloc_zeroed(Layout::from_size_align(task_local_pages.len(), 4096).unwrap()) }
                as usize;

        memory_map.insert(
            "Mutable Task Local Data".into(),
            RangeDescriptor {
                virtual_range: make_virtual(&binary_info.task_local),
                translation: Translation::Offset(
                    page_address,
                ),
                attribute_fields: USER_RW_,
                granule: Granule::Page4KiB,
            },
        );

        let mut my_memory_manager = TaskMemoryManager {
            memory_descriptors: memory_map,
            additional_table_hack: unsafe { Box::new_zeroed().assume_init() },
        };

        for (name, memory) in my_memory_manager.memory_descriptors.iter() {
            let step = match &memory.granule {
                Granule::Page4KiB => 1 << 12,
                Granule::Block2MiB => 1 << 21,
                Granule::Block1GiB => 1 << 30,
            };
            let offset = if let Translation::Offset(value) = memory.translation {
                value - memory.virtual_range.start
            } else {
                0
            };

            let range = memory.virtual_range.clone();
            for address in range.step_by(step) {
                unsafe {
                    my_memory_manager
                        .additional_table_hack
                        .map_memory(address, offset, &memory.attribute_fields, memory.granule)
                        .unwrap();
                }
            }
        }
        unsafe {
            match my_memory_manager
                .additional_table_hack
                .translate(0x1_0009_d030)
            {
                Ok(t) => crate::println!("Prev {:x} -> {:x}", 0x1_0009_d030u64, t as u64),
                Err(t) => crate::println!("Prev {:x} -> None", 0x1_0009_d030u64,),
            }
        }

        my_memory_manager
    }
}
