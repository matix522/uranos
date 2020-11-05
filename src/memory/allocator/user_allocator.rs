use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct UserAllocator;

unsafe impl GlobalAlloc for UserAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!();
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unimplemented!();
    }
}
impl UserAllocator {
    pub const fn get() -> Self {
        UserAllocator {}
    }
}
