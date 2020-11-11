use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ops::Range;
use core::ptr::null_mut;

use super::block_descriptor::Block;

use crate::sync::mutex::Mutex;
pub struct KernelAllocator {
    control_block: UnsafeCell<ControlBlock>,
}

unsafe impl Sync for KernelAllocator {}

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
        if self.unused_blocks.is_null() {
            self.stack_top = self.stack_top.offset(-1);
            self.memory_size -= core::mem::size_of::<Block>();
            return self.stack_top;
        }

        let block = self.unused_blocks;
        self.unused_blocks = (*block).next;
        block
    }

    unsafe fn get_top_bytes(&mut self, size: usize) -> *mut u8 {
        let return_val = self.data_top;
        self.data_top = self.data_top.add(size);
        self.memory_size -= size;
        return_val
    }
    /// # Safety:
    /// It is assumed that pointer list is non_null
    unsafe fn find<Predicate: Fn(*mut Block) -> bool>(
        list: *mut Block,
        p: Predicate,
    ) -> (*mut Block, *mut Block) {
        let mut prev_free_list = list;
        let mut free_list = (*list).next;
        while !free_list.is_null() {
            if p(free_list) {
                return (prev_free_list, free_list);
            }
            prev_free_list = free_list;
            free_list = (*free_list).next;
        }
        (prev_free_list, free_list)
    }
    #[allow(dead_code)]
    unsafe fn for_each<Function: Fn(&mut Block)>(list: *mut Block, f: Function) {
        let mut free_list = (*list).next;
        while !free_list.is_null() {
            f(&mut *free_list);
            free_list = (*free_list).next;
        }
    }

    unsafe fn find_free_memory(&mut self, layout: Layout) -> *mut Block {
        let requested_size = core::cmp::max(8, layout.size());
        let requested_align = core::cmp::max(8, layout.align());

        let (mut prev_free, next_free) = ControlBlock::find(self.free_list, |next| {
            let next_block = &mut *next;
            let padding_size = next_block.data_ptr.align_offset(requested_align);
            let aligned_size = next_block.data_size - padding_size;
            aligned_size >= requested_size
        });

        if !next_free.is_null() {
            let padding_size = (*next_free).data_ptr.align_offset(requested_align);
            let aligned_ptr = (*next_free).data_ptr.add(padding_size);
            let aligned_size = (*next_free).data_size - padding_size;
            if padding_size > 0 {
                let padding_block = self.get_new_block();
                *padding_block = Block::new(next_free, (*next_free).data_ptr, padding_size);
                (*prev_free).next = padding_block;
                prev_free = padding_block;
            }
            if aligned_size > requested_size {
                let new_free_block = self.get_new_block();

                *new_free_block = Block::new(
                    (*next_free).next,
                    aligned_ptr.add(requested_size),
                    aligned_size - requested_size,
                );
                (*next_free).next = new_free_block
            }
            (*prev_free).next = (*next_free).next;

            (*next_free).data_ptr = aligned_ptr;
            (*next_free).data_size = requested_size;

            let (previous_alloc, next_alloc) =
                ControlBlock::find(self.alloc_list, |next| (*next).data_ptr > aligned_ptr);
            (*previous_alloc).next = next_free;
            (*next_free).next = next_alloc;

            return next_free;
        }

        let aligned_offset = self.data_top.align_offset(requested_align);

        if self.memory_size - aligned_offset < requested_size {
            return null_mut();
        }

        if aligned_offset > 0 {
            let new_free_block = self.get_new_block();
            *new_free_block = Block::new(
                null_mut(),
                self.get_top_bytes(aligned_offset),
                aligned_offset,
            );

            let (previous, next) = ControlBlock::find(self.free_list, |next| {
                (*next).data_ptr > (*new_free_block).data_ptr
            });

            (*previous).next = new_free_block;
            (*new_free_block).next = next;
        }
        let allocated_data_ptr = self.get_top_bytes(requested_size);
        let new_allocated_block = self.get_new_block();

        let (previous, next) = ControlBlock::find(self.alloc_list, |next| {
            (*next).data_ptr > allocated_data_ptr
        });

        *new_allocated_block = Block::new(next, allocated_data_ptr, requested_size);
        (*previous).next = new_allocated_block;
        new_allocated_block
    }
}

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
    pub unsafe fn initialize_memory(&self, range: Range<usize>) {
        let control = &mut *self.control_block.get();
        assert!(range.end % 8 == 0);
        assert!(range.start % 8 == 0);

        control.stack_top = range.end as *mut _;
        control.data_top = range.start as *mut _;
        control.memory_size = range.end - range.start;

        let free_warden = control.stack_top.offset(-1);
        let alloc_warden = control.stack_top.offset(-2);

        *free_warden = core::mem::zeroed();
        *alloc_warden = core::mem::zeroed();

        control.alloc_list = alloc_warden;
        control.free_list = free_warden;

        control.stack_top = control.stack_top.offset(-2);
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let control = &mut *self.control_block.get();

        let allocated_block = control.find_free_memory(layout);

        if !allocated_block.is_null() {
            if crate::config::debug_alloc() {
                crate::println!(
                    "Alloc: {:x} Layout {{size: {:x}, align : {:x} }} ",
                    (*allocated_block).data_ptr as u64,
                    layout.size(),
                    layout.align()
                );
            }
            return (*allocated_block).data_ptr;
        }

        null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let control = &mut *self.control_block.get();

        if crate::config::debug_alloc() {
            crate::println!(
                "Dealloc: {:x} Layout {{size: {:x}, align : {:x} }} ",
                ptr as u64,
                _layout.size(),
                _layout.align()
            );
        }
        let (previous_alloc, element) =
            ControlBlock::find(control.alloc_list, |next| (*next).data_ptr == ptr);
        if element.is_null() {
            crate::println!(
                "\x1b[31m[WARN] Could not deallocate ptr {:x}\x1b[0m",
                ptr as u64
            );
            return;
        }

        // remove element from alloc list
        (*previous_alloc).next = (*element).next;

        let (previous_free, next_free) =
            ControlBlock::find(control.free_list, |next| (*next).data_ptr > ptr);
        (*previous_free).next = element;
        (*element).next = next_free;

        let previous = &mut *previous_free;
        let current = &mut *element;

        if previous.data_ptr.add(previous.data_size) == current.data_ptr {
            previous.data_size += current.data_size;
            previous.next = next_free;
            current.data_ptr = null_mut();
            current.data_size = 0;
            current.next = control.unused_blocks;
            control.unused_blocks = element;

            if !next_free.is_null()
                && previous.data_ptr.add(previous.data_size) == (*next_free).data_ptr
            {
                let next = &mut *next_free;

                previous.data_size += next.data_size;
                previous.next = next.next;

                next.data_ptr = null_mut();
                next.data_size = 0;
                next.next = control.unused_blocks;
                control.unused_blocks = next_free;
            }
        } else if !next_free.is_null()
            && current.data_ptr.add(current.data_size) == (*next_free).data_ptr
        {
            let next = &mut *next_free;

            current.data_size += next.data_size;
            current.next = next.next;

            next.data_ptr = null_mut();
            next.data_size = 0;
            next.next = control.unused_blocks;
            control.unused_blocks = next_free;
        }
    }
}

#[link_section = ".heap"]
pub static ALLOCATOR: Mutex<KernelAllocator> = Mutex::new(KernelAllocator::new());
