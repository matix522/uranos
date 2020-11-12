pub struct Block {
    pub next: *mut Block,
    pub data_ptr: *mut u8,
    pub data_size: usize,
}

impl Block {
    pub const fn new(next: *mut Block, data_ptr: *mut u8, data_size: usize) -> Self {
        Block {
            next,
            data_ptr,
            data_size,
        }
    }
}
