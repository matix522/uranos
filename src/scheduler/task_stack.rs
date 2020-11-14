use alloc::alloc::*;

unsafe impl Sync for TaskStack {}

#[derive(Debug)]
pub struct TaskStack {
    ptr: *mut u8,
    virtual_address: Option<usize>,
    size: usize,
    is_kernel: bool,
}

use crate::memory::armv8::mmu::translate;
use crate::memory::memory_controler::{
    map_kernel_memory, map_user_memory, unmap_kernel_memory, unmap_user_memory, AddressSpace,
};

impl TaskStack {
    pub fn new(size: usize, virtual_address: Option<usize>, is_kernel: bool) -> Option<Self> {
        let layout = Layout::from_size_align(size, 4096).ok()?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return None;
        }
        if let Some(v_address) = virtual_address {
            if is_kernel {
                map_kernel_memory(
                    &format!("Stack_memory_el1_{:x}", v_address),
                    v_address..v_address + size,
                    ptr as usize,
                    true,
                );
            } else {
                map_user_memory(
                    &format!("Stack_memory_el0_{:x}", v_address),
                    v_address..v_address + size,
                    ptr as usize,
                    true,
                );
            }
            if crate::config::debug_mmu() {
                let v_ptr =
                    (v_address | if is_kernel { crate::KERNEL_OFFSET } else { 0 }) as *const u64;

                unsafe {
                    crate::println!("ACCESS TEST: {:x}", *v_ptr);
                }
            }
        }
        Some(TaskStack {
            ptr,
            virtual_address,
            size,
            is_kernel,
        })
    }

    pub fn base(&self) -> usize {
        if let Some(v_address) = self.virtual_address {
            v_address + self.size - 16 | if self.is_kernel { crate::KERNEL_OFFSET } else { 0 }
        } else {
            self.ptr as usize + self.size - 16
        }
    }
    pub fn top(&self) -> usize {
        if let Some(v_address) = self.virtual_address {
            v_address | if self.is_kernel { crate::KERNEL_OFFSET } else { 0 }
        } else {
            self.ptr as usize
        }
    }
    pub fn size(&self) -> usize {
        self.size
    }
}
impl Drop for TaskStack {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, 16).unwrap();
        if let Some(v_address) = self.virtual_address {
            if self.is_kernel {
                unmap_kernel_memory(&format!("Stack_memory_el1_{:x}", v_address));
            } else {
                unmap_user_memory(&format!("Stack_memory_el0_{:x}", v_address));
            }
        }
        unsafe { dealloc(self.ptr, layout) };
    }
}

// 100100 10 0000 0000   0000 0000 01 000110
