use alloc::alloc::*;

unsafe impl Sync for TaskStack {}

#[derive(Debug)]
pub struct TaskStack {
    ptr: *mut u8,
    size: usize,
}
impl TaskStack {
    pub fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, 16).ok()?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return None;
        }
        Some(TaskStack { ptr, size })
    }
    pub fn stack_base(&self) -> usize {
        self.ptr as usize + self.size - 16
    }
    pub fn stack_top(&self) -> usize {
        self.ptr as usize
    }
    pub fn size(&self) -> usize {
        self.size
    }
}
impl Drop for TaskStack {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, 16).unwrap();
        unsafe { dealloc(self.ptr, layout) };
    }
}
