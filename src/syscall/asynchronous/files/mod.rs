pub mod open;

pub const ONLY_MSB_OF_USIZE: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

pub enum AsyncFileDescriptor{
    FileDescriptor(usize),
    AsyncSyscallReturnValue(usize),
}

impl AsyncFileDescriptor{
    pub fn to_usize(&self) -> usize{
        match self {
            AsyncFileDescriptor::FileDescriptor(val) => val & !ONLY_MSB_OF_USIZE,
            AsyncFileDescriptor::AsyncSyscallReturnValue(val) => val | ONLY_MSB_OF_USIZE,
        }
    } 
}