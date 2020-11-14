use core::alloc::{GlobalAlloc, Layout};

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
    #[allow(dead_code)]
    pub const fn get() -> Self {
        UserAllocator {}
    }
}
