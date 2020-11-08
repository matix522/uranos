use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::size_of;
use core::ops::Range;
use core::ptr::null_mut;

use super::block_descriptor::Block;

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
            // crate::println!("NEW BLOCK: {:x}", self.stack_top as u64);
            return self.stack_top;
        }
        // crate::println!("USED BLOCK: {:x}", self.unused_blocks as u64);

        let block = self.unused_blocks;
        self.unused_blocks = (*block).next;
        block
    }

    unsafe fn get_top_bytes(&mut self, size: usize) -> *mut u8 {
        // crate::println!("DATA TOP: {:x}", self.data_top as u64);

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

    unsafe fn for_each<Function: Fn(&mut Block)>(list: *mut Block, f: Function) {
        let mut prev_free_list = list;
        let mut free_list = (*list).next;
        while !free_list.is_null() {
            f(&mut *free_list);
            prev_free_list = free_list;
            free_list = (*free_list).next;
        }
    }

    unsafe fn find_free_memory(&mut self, layout: Layout) -> *mut Block {
        let requested_size = core::cmp::max(8, layout.size());
        let requested_allign = core::cmp::max(8, layout.align());

        let (mut prev_free, next_free) = ControlBlock::find(self.free_list, |next| {
            let next_block = &mut *next;
            let padding_size = next_block.data_ptr.align_offset(requested_allign);
            let alligned_size = next_block.data_size - padding_size;
            alligned_size >= requested_size
        });

        // crate::println!("control.free_list  - {:x}", self.free_list as usize);
        // crate::println!("control.alloc_list - {:x}", self.alloc_list as usize);


        // ControlBlock::for_each(self.free_list, |link| {
        //     crate::println!("********* FREE *********");
        //     crate::println!("Block Info - {:x}", link as *mut _ as usize);

        //     crate::println!("Start      - {:x}", link.data_ptr as usize);
        //     crate::println!("Size       - {:x}", link.data_size);
        //     crate::println!("End        - {:x}", link.data_ptr as usize + link.data_size);
        //     crate::println!("************************");

        // });
        // ControlBlock::for_each(self.alloc_list, |link| {
        //     crate::println!("********* ALLOC ********");
        //     crate::println!("Block Info - {:x}", link as *mut _ as usize);

        //     crate::println!("Start      - {:x}", link.data_ptr as usize);
        //     crate::println!("Size       - {:x}", link.data_size);
        //     crate::println!("End        - {:x}", link.data_ptr as usize + link.data_size);
        //     crate::println!("************************");

        // });

        // crate::println!("next_free: {:x}", next_free as u64);
        // crate::println!("data_top: {:x}", self.data_top as u64);

        if !next_free.is_null() {
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

            (*next_free).data_ptr = alligned_ptr;
            (*next_free).data_size = requested_size;

            let (previous_alloc, next_alloc) = ControlBlock::find(self.alloc_list, |next| {
                (*next).data_ptr > alligned_ptr
            });
            (*previous_alloc).next = next_free;
            (*next_free).next = next_alloc;

            return next_free;
        }

        let alligned_offset = self.data_top.align_offset(requested_allign);

        if self.memory_size - alligned_offset < requested_size {
            return null_mut();
        }

        if alligned_offset > 0 {
            let new_free_block = self.get_new_block();
            *new_free_block = Block::new(
                null_mut(),
                self.get_top_bytes(alligned_offset),
                alligned_offset,
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
        // let base_address = align_address(control as *const _ as usize , config::page_size());
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
                crate::println!("Alloc: {:x} Layout {{size: {:x}, allign : {:x} }} ", (*allocated_block).data_ptr as u64, layout.size(), layout.align());
            }
            return (*allocated_block).data_ptr;
        }

        null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let control = &mut *self.control_block.get();

        // ControlBlock::for_each(control.free_list, |link| {
        //     crate::println!("********* FREE *********");
        //     crate::println!("Block Info - {:x}", link as *mut _ as usize);

        //     crate::println!("Start      - {:x}", link.data_ptr as usize);
        //     crate::println!("Size       - {:x}", link.data_size);
        //     crate::println!("End        - {:x}", link.data_ptr as usize + link.data_size);
        //     crate::println!("************************");

        // });
        // crate::println!("\nPtr - {:x}", ptr as usize);
        if crate::config::debug_alloc() {
            crate::println!("Dealloc: {:x} Layout {{size: {:x}, allign : {:x} }} ", ptr as u64, _layout.size(), _layout.align());
        }
        // ControlBlock::for_each(control.alloc_list, |link| {
        //     crate::println!("Start      - {:x}", link.data_ptr as usize);
        // });




        let control = &mut *self.control_block.get();
        let (previous, element) =
            ControlBlock::find(control.alloc_list, |next| (*next).data_ptr == ptr);
        if element.is_null() {
            crate::println!("[WARN] Could not deallocate ptr {:x}", ptr as u64);
        }

        // remove element from alloc list
        (*previous).next = (*element).next;

        let (previous_free, next_free) =
            ControlBlock::find(control.free_list, |next| (*next).data_ptr > ptr);
        (*previous_free).next = element;
        (*element).next = next_free;

        let previous = &mut *previous_free;
        let current = &mut *element;
        let next = &mut *next_free;

        if previous.data_ptr.add(previous.data_size) == current.data_ptr {
            previous.data_size += current.data_size;
            previous.next = next_free;

            current.data_ptr = null_mut();
            current.data_size = 0;
            current.next = control.unused_blocks;
            control.unused_blocks = element;

            if previous.data_ptr.add(previous.data_size) == next.data_ptr {
                previous.data_size += next.data_size;
                previous.next = next.next;

                next.data_ptr = null_mut();
                next.data_size = 0;
                next.next = control.unused_blocks;
                control.unused_blocks = next_free;
            }
        }
        if current.data_ptr.add(current.data_size) == next.data_ptr {
            current.data_size += next.data_size;
            current.next = next.next;

            next.data_ptr = null_mut();
            next.data_size = 0;
            next.next = control.unused_blocks;
            control.unused_blocks = next_free;
        }
    }
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
    pub(super) fn heap_end(&self) -> usize {
        self.heap_start() + self.heap_size
    }
    fn block_list(&self) -> *mut OldBlock {
        self.heap_start() as *mut OldBlock
    }
}
#[link_section = ".heap"]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::new();

// #[link_section = ".heap"]
// pub static ALLOCATOR: KernelAllocatorOld = KernelAllocatorOld::new(0x800_0000);

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
