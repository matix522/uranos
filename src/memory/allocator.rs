use core::alloc::{GlobalAlloc, Layout};
use core::ops::Range;
use core::ptr::null_mut;

mod block_descriptor;
mod kernel_allocator;
mod user_allocator;

use crate::boot::mode::ExceptionLevel;
pub struct ChooseAllocator;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: ChooseAllocator = ChooseAllocator;

unsafe fn get_level() -> ExceptionLevel {
    let level = crate::syscall::syscall0(crate::syscall::Syscalls::CheckEL as usize);
    match level {
        0 => ExceptionLevel::User,
        1 => ExceptionLevel::Kernel,
        2 => ExceptionLevel::Hypervisor,
        3 => ExceptionLevel::Firmware,
        _ => unreachable!(),
    }
}

unsafe impl GlobalAlloc for ChooseAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match get_level() {
            ExceptionLevel::Kernel => kernel_allocator::ALLOCATOR.alloc(layout),
            ExceptionLevel::User => user_allocator::UserAllocator::get().alloc(layout),
            _ => null_mut(),
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match get_level() {
            ExceptionLevel::Kernel => kernel_allocator::ALLOCATOR.dealloc(ptr, layout),
            ExceptionLevel::User => user_allocator::UserAllocator::get().dealloc(ptr, layout),
            _ => panic!("Global Allocator in invalid context!"),
        };
    }
}

pub fn kernel_heap_range() -> Range<usize> {
    let allocator = &kernel_allocator::ALLOCATOR;
    allocator.heap_start()..allocator.heap_end()
}

#[alloc_error_handler]
pub fn bad_alloc(layout: Layout) -> ! {
    crate::println!("bad_alloc: {:?}", layout);
    crate::aarch64::halt()
}
