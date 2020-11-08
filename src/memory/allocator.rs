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
        crate::println!("ALLOCATE: {:?}", layout);
        match get_level() {
            ExceptionLevel::Kernel => kernel_allocator::ALLOCATOR.alloc(layout),
            ExceptionLevel::User => user_allocator::UserAllocator::get().alloc(layout),
            _ => null_mut(),
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        crate::println!("DEALLOCATE: {:?}", layout);
        match get_level() {
            ExceptionLevel::Kernel => kernel_allocator::ALLOCATOR.dealloc(ptr, layout),
            ExceptionLevel::User => user_allocator::UserAllocator::get().dealloc(ptr, layout),
            _ => panic!("Global Allocator in invalid context!"),
        };
    }
}
pub unsafe fn init_kernel() {
    kernel_allocator::ALLOCATOR.initialize_memory(kernel_heap_range());
}

pub fn kernel_heap_range() -> Range<usize> {
    let allocator = &kernel_allocator::ALLOCATOR;
    let allocator_address =
        &kernel_allocator::ALLOCATOR as *const kernel_allocator::KernelAllocator as *const u8;
    let heap_start = unsafe { allocator_address.add(4096) };
    heap_start as usize..heap_start as usize + 0x1000_0000
}

#[alloc_error_handler]
pub fn bad_alloc(layout: Layout) -> ! {
    crate::println!("bad_alloc: {:?}", layout);
    crate::aarch64::halt()
}
