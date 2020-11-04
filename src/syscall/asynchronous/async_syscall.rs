use crate::utils::circullar_buffer::*;
use num_traits::{FromPrimitive};

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum AsyncSyscalls{
    Print,
}

pub struct AsyncSyscall<'a>{
    pub data_size: usize,
    pub syscall_type: AsyncSyscalls,
    pub data: &'a[u8],
}

pub struct AsyncSyscallReturnedValue<'a>{
    pub syscall_type: AsyncSyscalls,
    pub data: ReturnedValue<'a>,
}

pub fn send_async_syscall(buffer: &mut CircullarBuffer, syscall: AsyncSyscall){
    let usize_size = core::mem::size_of::<usize>();
    let mut buffer_frame = buffer.reserve(usize_size + syscall.data_size).expect("Error during sending async syscall");
    unsafe{
        *(&mut *buffer_frame as *mut _ as *mut usize) = syscall.syscall_type as usize;
        core::ptr::copy_nonoverlapping(syscall.data as *const _ as *const u8 , (&mut (*buffer_frame) as *mut _ as *mut u8).add(usize_size), syscall.data_size);
    }
}

pub fn read_async_syscall(buffer: &mut CircullarBuffer) -> Option<AsyncSyscallReturnedValue>{
    if buffer.is_empty(){
        return None;
    }
    let buffer_entry: ReturnedValue = buffer.get_value().expect("Error during reading async syscall");
    unsafe {
        let syscall_type_usize = *(buffer_entry.get_ref() as *const _ as *const usize);
        let syscall_type = AsyncSyscalls::from_usize(syscall_type_usize).unwrap();
        Some(AsyncSyscallReturnedValue{
            syscall_type,
            data: buffer_entry,
        })
    }
    
}
