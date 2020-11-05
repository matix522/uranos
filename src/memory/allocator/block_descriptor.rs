use core::mem::size_of;

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

pub struct OldBlock {
    pub next: *mut OldBlock,
    pub data_size: usize,
}
unsafe impl Send for OldBlock {}
unsafe impl Sync for OldBlock {}
impl OldBlock {
    pub fn size_of(&self) -> usize {
        size_of::<Self>() + self.data_size
    }
}

impl core::fmt::Display for OldBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "****************************")?;
        writeln!(f, "*Start:  {:#018x}*", self as *const Self as u64)?;
        writeln!(
            f,
            "*Ptr:    {:#018x}*",
            self as *const Self as usize + size_of::<Self>()
        )?;
        writeln!(f, "*D Size: {:#018x}*", self.data_size)?;
        writeln!(
            f,
            "*End:    {:#018x}*",
            self as *const Self as usize + self.size_of()
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
