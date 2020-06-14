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

pub struct SystemAllocator {
    heap_size: usize,
    first_block: core::cell::UnsafeCell<Block>,
}
impl core::fmt::Display for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "****************************")?;
        writeln!(f, "*Start:  {:#018x}*", self as *const Self as u64)?;
        writeln!(f, "*D Size: {:#018x}*", self.data_size)?;
        writeln!(
            f,
            "*End:    {:#018x}*",
            self as *const Self as usize + self.size_of()
        )?;
        writeln!(
            f,
            "*Ptr:    {:#018x}*",
            self as *const Self as usize + size_of::<Self>()
        )?;

        if self.next.is_null() {
            writeln!(f, "*Next:        NULL         *")?;
        } else {
            writeln!(f, "*Next:   {:#018x}*", self.next as u64)?;
        }

        write!(f, "****************************")?;
        Ok(())
    }
}
impl core::fmt::Display for SystemAllocator {
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
unsafe impl Sync for SystemAllocator {}
impl SystemAllocator {
    fn heap_start(&self) -> usize {
        self.first_block.get() as usize
    }
    fn heap_end(&self) -> usize {
        self.heap_start() + self.heap_size
    }
    fn block_list(&self) -> *mut Block {
        self.heap_start() as *mut Block
    }
}
#[global_allocator]
#[link_section = ".heap"]
pub static A: SystemAllocator = SystemAllocator::new(0x400_0000);

pub fn heap_start() -> usize {
    A.heap_start()
}
pub fn heap_end() -> usize {
    A.heap_end()
}

unsafe fn is_the_space_big_enough(
    base_address: *mut Block,
    required_layout: Layout,
    end_address: usize,
) -> bool {
    let potential_address = align_address(
        base_address as usize + (*base_address).size_of() + size_of::<Block>(),
        required_layout.align(),
    );
    potential_address + required_layout.size() <= end_address
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
unsafe impl GlobalAlloc for SystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut previous = self.block_list();
        let mut current = (*previous).next;
        crate::println!("{:?}", layout);

        let size = layout.size();
        while !current.is_null() {
            if is_the_space_big_enough(previous, layout, current as usize) {
                // FOUND PLACE
                let mut new_block =
                    (align_address(previous as usize + (*previous).size_of(), layout.align())
                        - size_of::<Block>()) as *mut Block;
                (*new_block).next = current;
                (*new_block).data_size = size;
                (*previous).next = new_block;
                let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;

                return ptr;
            }
            previous = current;
            current = (*current).next;
        }

        if is_the_space_big_enough(previous, layout, self.heap_end()) {
            // FOUND PLACE
            let mut new_block =
                (align_address(previous as usize + (*previous).size_of(), layout.align())
                    - size_of::<Block>()) as *mut Block;
            crate::println!("{:#018x}", new_block as u64);

            (*new_block).next = null_mut();
            (*new_block).data_size = size;
            (*previous).next = new_block;
            let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;

            return ptr;
        }
        null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // Every pointer returned by alloc is allignred at least to aligment of Block
        #[allow(clippy::cast_ptr_alignment)]
        let block = ptr.offset(-(size_of::<Block>() as isize)) as *mut Block;
        #[deny(clippy::cast_ptr_alignment)]
        let mut previous = self.block_list();

        let mut current = (*previous).next;
        // TOTALY UNSAFE FOR NOW
        while current != block && !current.is_null() && (current as u64) < (block as u64) {
            previous = current;
            current = (*current).next;
        }

        if !current.is_null() {
            (*previous).next = (*current).next;
        }
    }
}

impl SystemAllocator {
    pub const fn new(heap_size: u64) -> Self {
        SystemAllocator {
            heap_size: heap_size as usize,
            first_block: UnsafeCell::new(Block {
                next: null_mut(),
                data_size: 0,
            }),
        }
    }
}

#[alloc_error_handler]
pub fn bad_alloc(layout: core::alloc::Layout) -> ! {
    crate::println!("bad_alloc: {:?}", layout);
    crate::aarch64::halt()
}
