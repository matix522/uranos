use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::size_of;
use core::ops::Range;
use core::ptr::null_mut;

use crate::config;

use super::block_descriptor::Block;

pub struct KernelAllocator {
    control_block: UnsafeCell<ControlBlock>,
}
struct ControlBlock {
    free_list: *mut Block,
    alloc_list: *mut Block,
    unused_blocks: *mut Block,
    stack_top: *mut Block,
    data_top: *mut u8,
    memory_size: usize,
}

impl ControlBlock {
    unsafe fn get_new_block(&mut self) -> *mut Block {
        self.stack_top = self.stack_top.offset(-1);
        self.stack_top
    }
    /// # Safety:
    /// It is assumed that pointer list is non_null
    unsafe fn find<Predicate: Fn(*mut Block) -> bool>(
        list: *mut Block,
        p: Predicate,
    ) -> (*mut Block, *mut Block) {
        let mut prev_free_list = list;
        let mut free_list = (*list).next;
        while free_list != null_mut() {
            if !p(free_list) {
                return (prev_free_list, free_list);
            }
        }
        return (prev_free_list, free_list);
    }

    unsafe fn find_free_memory(&mut self, layout: Layout) -> *mut Block {
        let requested_size = core::cmp::max(8, layout.size());
        let requested_allign = core::cmp::max(8, layout.align());

        let (mut prev_free, next_free) = ControlBlock::find(self.free_list, |next| {
            let next_block = &mut *next;
            let padding_size = next_block.data_ptr.align_offset(requested_allign);
            let alligned_ptr = next_block.data_ptr.add(padding_size);
            let alligned_size = next_block.data_size - padding_size;
            alligned_size >= requested_size
        });
        if next_free != null_mut() {
            let padding_size = (*next_free).data_ptr.align_offset(requested_allign);
            let alligned_ptr = (*next_free).data_ptr.add(padding_size);
            let alligned_size = (*next_free).data_size - padding_size;
            if padding_size > 0 {
                let padding_block = self.get_new_block();
                *padding_block = Block::new(next_free, (*next_free).data_ptr, padding_size);
                (*prev_free).next = padding_block;
                prev_free = padding_block;
            }
            if alligned_size > requested_size {
                let new_free_block = self.get_new_block();

                *new_free_block = Block::new(
                    (*next_free).next,
                    alligned_ptr.add(requested_size),
                    alligned_size - requested_size,
                );
                (*next_free).next = new_free_block
            }
            (*prev_free).next = (*next_free).next;
            return next_free;
        }

        let alligned_offset = self.data_top.align_offset(requested_allign);

        if alligned_offset > 0 {
            let new_free_block = self.get_new_block();
            (*new_free_block).data_size = alligned_offset;
            (*new_free_block).data_ptr = self.data_top;
            (*new_free_block).next = null_mut();

            let (previous, next) = ControlBlock::find(self.free_list, |next| {
                (*next).data_ptr < (*new_free_block).data_ptr
            });

            (*previous).next = new_free_block;
            (*new_free_block).next = next;
        }
        null_mut()
    }
}
// let mut prev_free_list = self.free_list;
        // let mut free_list = (*self.free_list).next;

        // while free_list != null_mut() {
        //     let free_block = &mut *free_list;
        //     let padding_size = free_block.data_ptr.align_offset(requested_allign);
        //     let alligned_ptr = free_block.data_ptr.add(padding_size);
        //     let alligned_size = free_block.data_size - padding_size;
        //     if alligned_size >= requested_size {
        //         if padding_size > 0 {
        //             let padding_block = self.get_new_block();
        //             *padding_block = Block::new(free_list, free_block.data_ptr, padding_size);
        //             (*prev_free_list).next = padding_block;
        //             prev_free_list = padding_block;
        //         }
        //         if alligned_size > requested_size {
        //             let new_free_block = self.get_new_block();

        //             *new_free_block = Block::new(
        //                 free_block.next,
        //                 alligned_ptr.add(requested_size),
        //                 alligned_size - requested_size,
        //             );
        //             free_block.next = new_free_block
        //         }
        //         (*prev_free_list).next = free_block.next;
        //         return free_block as *mut Block;
        //     }
        //     prev_free_list = free_list;
        //     free_list = (*free_list).next;
        // }


impl KernelAllocator {
    pub const fn new() -> Self {
        KernelAllocator {
            control_block: UnsafeCell::new(ControlBlock {
                free_list: null_mut(),
                alloc_list: null_mut(),
                unused_blocks: null_mut(),
                stack_top: null_mut(),
                data_top: null_mut(),
                memory_size: 0,
            }),
        }
    }
    pub unsafe fn initialize_memory(&mut self, range: Range<usize>) {
        let control = &mut *self.control_block.get();
        // let base_address = align_address(control as *const _ as usize , config::page_size());
        assert!(range.end % 8 == 0);
        assert!(range.start % 8 == 0);

        control.stack_top = range.end as *mut _;
        control.data_top = range.start as *mut _;
        control.memory_size = range.end - range.start;

        let free_warden = control.stack_top.offset(-1);
        let alloc_warden = control.stack_top.offset(-2);

        *free_warden = unsafe { core::mem::zeroed() };
        *alloc_warden = unsafe { core::mem::zeroed() };

        control.stack_top = control.stack_top.offset(-2);
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let control = &mut *self.control_block.get();

        null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {}
}

use super::block_descriptor::OldBlock;

pub struct KernelAllocatorOld {
    heap_size: usize,
    first_block: core::cell::UnsafeCell<OldBlock>,
}

impl core::fmt::Display for KernelAllocatorOld {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Start Address : {:#018x}", self.heap_start())?;
        writeln!(f, "End Address :   {:#018x}", self.heap_end())?;
        writeln!(f, "Size :          {:#018x}", self.heap_size)?;

        let mut block = self.block_list();
        unsafe {
            while !block.is_null() {
                writeln!(f, "{}", *block)?;
                block = (*block).next
            }
        }
        Ok(())
    }
}
unsafe impl Sync for KernelAllocatorOld {}
impl KernelAllocatorOld {
    pub(super) fn heap_start(&self) -> usize {
        self.first_block.get() as usize
    }
    fn heap_base(&self) -> usize {
        &self.heap_size as *const _ as usize
    }
    pub(super) fn heap_end(&self) -> usize {
        self.heap_start() + self.heap_size
    }
    fn block_list(&self) -> *mut OldBlock {
        self.heap_start() as *mut OldBlock
    }
}
#[link_section = ".heap"]
pub static ALLOCATOR: KernelAllocatorOld = KernelAllocatorOld::new(0x800_0000);

pub fn heap_start() -> usize {
    ALLOCATOR.heap_start()
}
pub fn heap_end() -> usize {
    ALLOCATOR.heap_end()
}
pub fn heap_base() -> usize {
    ALLOCATOR.heap_base()
}
///
/// # Safety
/// aligment must be non 0
unsafe fn align_address(address: usize, aligment: usize) -> usize {
    if address % aligment == 0 {
        address
    } else {
        (address / aligment + 1) * aligment
    }
}
unsafe impl GlobalAlloc for KernelAllocatorOld {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut previous = self.block_list();
        let mut current = (*previous).next;
        // crate::println!("{}", self);
        while !current.is_null() {
            let end_of_previous = previous as usize + size_of::<OldBlock>() + (*previous).data_size;
            let potenital_address =
                align_address(end_of_previous + size_of::<OldBlock>(), layout.align());
            if potenital_address + layout.size() < current as usize {
                let block_base = (potenital_address - size_of::<OldBlock>()) as *mut OldBlock;
                (*block_base).next = current;
                (*block_base).data_size = layout.size();
                (*previous).next = block_base;

                return potenital_address as *mut u8;
            }
            previous = current;
            current = (*current).next;
        }
        let end_of_previous = previous as usize + size_of::<OldBlock>() + (*previous).data_size;
        let potenital_address =
            align_address(end_of_previous + size_of::<OldBlock>(), layout.align());
        if potenital_address + layout.size() < self.heap_end() as usize {
            let block_base = (potenital_address - size_of::<OldBlock>()) as *mut OldBlock;
            (*block_base).next = null_mut();
            (*block_base).data_size = layout.size();
            (*previous).next = block_base;

            return potenital_address as *mut u8;
        }
        // crate::println!("{}", self);
        null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // Every pointer returned by alloc is allignred at least to aligment of OldBlock
        #[allow(clippy::cast_ptr_alignment)]
        let block = ptr.offset(-(size_of::<OldBlock>() as isize)) as *mut OldBlock;
        #[deny(clippy::cast_ptr_alignment)]
        let mut previous = self.block_list();

        let mut current = (*previous).next;

        while current != block && !current.is_null() {
            //&& (current as u64) < (block as u64) {
            previous = current;
            current = (*current).next;
        }

        if !current.is_null() {
            (*previous).next = (*current).next;
        }
    }
}

impl KernelAllocatorOld {
    pub const fn new(heap_size: u64) -> Self {
        KernelAllocatorOld {
            heap_size: heap_size as usize - size_of::<usize>(),
            first_block: UnsafeCell::new(OldBlock {
                next: null_mut(),
                data_size: 10,
            }),
        }
    }
}
