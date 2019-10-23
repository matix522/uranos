use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem::size_of;
use core::ptr::null_mut;
pub struct Block {
    next: *mut Block,
    data_size: usize,
}
unsafe impl Send for Block {}
unsafe impl Sync for Block {}
impl Block {
    pub fn size_of(&self) -> usize {
        size_of::<Self>() + self.data_size
    }
}

pub struct MainAllocator {
    heap_size: usize,
    first_block: Block,
}
impl MainAllocator {
    fn heap_start(&self) -> usize {
        return self as *const Self as usize + size_of::<usize>();
    }
    fn heap_end(&self) -> usize {
        return self.heap_start() + self.heap_size;
    }
    fn block_list(&self) -> *mut Block {
        return self.heap_start() as *mut Block;
    }
}
#[global_allocator]
#[link_section = ".heap"]
pub static A: MainAllocator = MainAllocator::new(0x100_0000);

pub fn heap_start() -> usize {
    return A.heap_start();
}
pub fn heap_end() -> usize {
    return A.heap_end();
}

unsafe fn is_the_space_big_enough(
    base_address: *mut Block,
    required_size: usize,
    end_address: usize,
) -> bool {
    base_address as usize
        + align_address((*base_address).size_of(), 8)
        + required_size
        + size_of::<Block>()
        <= end_address
}
///
/// # Safety
/// aligment must be non 0
unsafe fn align_address(address: usize, aligment: usize) -> usize {
    return if address % aligment == 0 {
        address
    } else {
        (address / aligment + 1) * aligment
    };
}
unsafe impl GlobalAlloc for MainAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut previous = self.block_list();
        let mut current = (*previous).next;

        let size = layout.size();

        while current != null_mut() {
            if is_the_space_big_enough(previous, size, current as usize) {
                // FOUND PLACE
                let mut new_block =
                    align_address(previous as usize + (*previous).size_of(), 8) as *mut Block;
                (*new_block).next = current;
                (*new_block).data_size = size;
                (*previous).next = new_block;
                let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;
                // crate::println!("sizeof block {:x}", size_of::<Block>() as usize);
                // crate::println!("alloc {:?} ", layout);
                // crate::println!("prev {:x} ", previous as u64);
                // crate::println!("new_block {:x} ", new_block as u64);
                // crate::println!("ptr {:x} ", ptr as u64);
                // crate::println!("next {:x} ", current as u64);
                return ptr;
            }
            previous = current;
            current = (*current).next;
        }
        if is_the_space_big_enough(previous, size, self.heap_end()) {
            // FOUND PLACE
            let mut new_block =
                align_address(previous as usize + (*previous).size_of(), 8) as *mut Block;
            (*new_block).next = null_mut();
            (*new_block).data_size = size;
            (*previous).next = new_block;
            let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;
            // crate::println!("alloc {:?} ", layout);
            // crate::println!("prev {:x} ", previous as u64);
            // crate::println!("new_block {:x} ", new_block as u64);
            // crate::println!("ptr {:x} ", ptr as u64);
            // crate::println!("next null");

            return ptr;
        }
        // ERROR_OOM
        // crate::println!("alloc {:?} ", layout);

        // crate::println!("ptr null");

        return null_mut();
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let block = ptr.offset(-(size_of::<Block>() as isize)) as *mut Block;
        // crate::println!("free {:?}: {:x}", layout, block as u64);

        let mut previous = self.block_list();
        let mut current = (*previous).next;
        // TOTALY UNSAFE FOR NOW
        while current != block {
            previous = current;
            current = (*current).next;
        }
        if current != null_mut() {
            (*previous).next = (*current).next;
        }

        // crate::println!("freed");
    }
}

impl MainAllocator {
    pub const fn new(heap_size: u64) -> Self {
        MainAllocator {
            heap_size: heap_size as usize,
            first_block: Block {
                next: null_mut(),
                data_size: 0,
            },
        }
    }
}

#[alloc_error_handler]
pub fn bad_alloc(layout: core::alloc::Layout) -> ! {
    crate::println!("bad_alloc: {:?}", layout);
    aarch64::halt();
}
