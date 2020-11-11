pub mod open;
pub mod read;

pub const ONLY_MSB_OF_USIZE: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

pub enum AsyncFileDescriptor {
    FileDescriptor(usize),
    AsyncSyscallReturnValue(usize),
}

impl AsyncFileDescriptor {
    pub fn from_usize(val: usize) -> Self {
        if val & ONLY_MSB_OF_USIZE > 0 {
            AsyncFileDescriptor::AsyncSyscallReturnValue(val & !ONLY_MSB_OF_USIZE)
        } else {
            AsyncFileDescriptor::FileDescriptor(val)
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            AsyncFileDescriptor::FileDescriptor(val) => val & !ONLY_MSB_OF_USIZE,
            AsyncFileDescriptor::AsyncSyscallReturnValue(val) => val | ONLY_MSB_OF_USIZE,
        }
    }
}
