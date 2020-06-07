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
        // writeln!(f, "0x0000000000000000");
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

        if (self.next == null_mut()) {
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
            while block != null_mut() {
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
        return self.first_block.get() as usize;
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
pub static A: SystemAllocator = SystemAllocator::new(0x100_0000);

pub fn heap_start() -> usize {
    return A.heap_start();
}
pub fn heap_end() -> usize {
    return A.heap_end();
}

unsafe fn is_the_space_big_enough(
    base_address: *mut Block,
    required_layout: Layout,
    end_address: usize,
) -> bool {
    let base =
        base_address as usize + align_address((*base_address).size_of(), required_layout.align());
    let required_size = required_layout.size() + size_of::<Block>();
    let alligned_end = align_address(base + required_size, required_layout.align());
    alligned_end <= end_address
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
unsafe impl GlobalAlloc for SystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut previous = self.block_list();
        let mut current = (*previous).next;

        // crate::print!("\x1b[33;5m");
        // crate::println!("{}",*self);
        // crate::print!("\x1b[0m");

        let size = layout.size();
        // crate::println!("alloc");
        while current != null_mut() {
            if is_the_space_big_enough(previous, layout, current as usize) {
                // FOUND PLACE
                let mut new_block =
                    align_address(previous as usize + (*previous).size_of(), layout.align())
                        as *mut Block;
                (*new_block).next = current;
                (*new_block).data_size = size;
                (*previous).next = new_block;
                let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;
                // crate::print!("\x1b[32;5m");
                // crate::println!("===========================");
                // crate::println!("{}", *new_block);
                // crate::println!("===========================");
                // crate::print!("\x1b[0m");

                // crate::print!("\x1b[36;5m");
                // crate::println!("{}",*self);
                // crate::print!("\x1b[0m");

                return ptr;
            }
            previous = current;
            current = (*current).next;
        }
        // crate::println!("end");

        if is_the_space_big_enough(previous, layout, self.heap_end()) {
            // FOUND PLACE
            let mut new_block =
                align_address(previous as usize + (*previous).size_of(), 8) as *mut Block;
            // crate::println!("prev: {:#018x} block: {:#018x}", previous as u64, new_block as u64);
            (*new_block).next = null_mut();
            (*new_block).data_size = size;
            (*previous).next = new_block;
            let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;
            //  crate::println!("prev: {:#018x} block: {:#018x}", previous as u64, new_block as u64);
            //     crate::print!("\x1b[32;5m");
            //         crate::println!("===========================");
            //         crate::println!("{}", *new_block);
            //         crate::println!("===========================");
            //         crate::print!("\x1b[0m");
            //                         crate::print!("\x1b[36;5m");
            // crate::println!("{}",*self);
            // crate::print!("\x1b[0m");

            return ptr;
        }
        // crate::print!("\x1b[91;5m");
        // crate::println!("args: {:#018x} {:#018x} {:#018x}",previous as u64, size, self.heap_end());
        // crate::println!("===========================");
        // crate::println!("            NULL           ");
        // crate::println!("===========================");
        // crate::print!("\x1b[0m");

        // ERROR_OOM
        // crate::println!("alloc {:?} ", layout);

        // crate::println!("ptr null");
        // crate::println!("Null");
        return null_mut();
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let block = ptr.offset(-(size_of::<Block>() as isize)) as *mut Block;
        // crate::println!("free {:?}: {:x}", layout, block as u64);

        let mut previous = self.block_list();

        // crate::print!("\x1b[33;5m");
        // crate::println!("{}",*self);
        // // crate::println!("prev: {:#018x} block: {:#018x}", previous as u64, block as u64);
        // crate::print!("\x1b[91;5m");

        // crate::println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // crate::println!("{}", *block);
        // crate::println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!");
        // crate::print!("\x1b[0m");

        let mut current = (*previous).next;
        // TOTALY UNSAFE FOR NOW
        while current != block && current != null_mut() && (current as u64) < (block as u64) {
            previous = current;
            current = (*current).next;
        }
        // crate::println!("prev: {:#018x} current: {:#018x}", previous as u64, current as u64);

        if current != null_mut() {
            (*previous).next = (*current).next;
        }
        //         crate::print!("\x1b[36;5m");
        // crate::println!("{}",*self);
        // crate::print!("\x1b[0m");

        // crate::println!("freed");
    }
}

impl SystemAllocator {
    pub const fn new(heap_size: u64) -> Self {
        SystemAllocator {
            heap_size: heap_size as usize,
                first_block : UnsafeCell::new(Block {
                next: null_mut(),
                data_size: 0,
            }),
        }
    }
}

#[alloc_error_handler]
pub fn bad_alloc(layout: core::alloc::Layout) -> ! {
    crate::println!("bad_alloc: {:?}", layout);
    aarch64::halt();
}

// unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//     let mut previous = self.block_list();
//     let mut current = (*previous).next;

//     while current != null_mut() {
//         if is_the_space_big_enough(previous, layout, current as usize) {
//             // FOUND PLACE
//             let mut new_block =
//                 align_address(previous as usize + (*previous).size_of(), layout.align()) as *mut Block;
//             (*new_block).next = current;
//             (*new_block).data_size = layout.size();
//             (*previous).next = new_block;
//             let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;
//             return ptr;
//         }
//         previous = current;
//         current = (*current).next;
//     }

//     if is_the_space_big_enough(previous, layout, self.heap_end()) {
//         // FOUND PLACE
//         let mut new_block =
//             align_address(previous as usize + (*previous).size_of(), 8) as *mut Block;
//         (*new_block).next = null_mut();
//         (*new_block).data_size = layout.size();
//         (*previous).next = new_block;
//         let ptr = (new_block as usize + size_of::<Block>()) as *mut u8;
//         return ptr;
//     }
//     return null_mut();
// }
