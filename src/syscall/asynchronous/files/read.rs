use super::*;
use alloc::string::String;
use alloc::string::ToString;
use crate::syscall::asynchronous::future_async_syscall_result::FutureAsyncSyscallResult;
use crate::utils::circullar_buffer::*;
use crate::vfs;
use crate::syscall::asynchronous::async_syscall::*;



pub struct AsyncReadSyscallData{
    pub afd: usize,
    pub length: usize,
}

impl AsyncReadSyscallData{
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { crate::utils::struct_to_slice::any_as_u8_slice(self) }
    }
}


pub fn read(afd: AsyncFileDescriptor, length: usize, buffer: *mut u8, submission_buffer: &mut CircullarBuffer){
    // let data = 
}