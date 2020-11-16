use super::*;
use crate::syscall::asynchronous::async_returned_values::*;
use crate::syscall::asynchronous::async_syscall::*;
use crate::syscall::files::read::handle_read;
use crate::utils::circullar_buffer::*;
use crate::vfs;

pub struct AsyncReadSyscallData {
    pub afd: usize,
    pub length: usize,
    pub buffer: *mut u8,
}

impl AsyncReadSyscallData {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { crate::utils::struct_to_slice::any_as_u8_slice(self) }
    }
}

pub fn read(
    afd: &AsyncFileDescriptor,
    length: usize,
    buffer: *mut u8,
    id: usize,
    submission_buffer: &mut CircullarBuffer,
) -> AsyncOpenedFile {
    let data = AsyncReadSyscallData {
        afd: afd.to_usize(),
        length,
        buffer,
    };

    let bytes = data.as_bytes();

    let a: AsyncSyscall = AsyncSyscall {
        data: bytes,
        id,
        data_size: bytes.len(),
        syscall_type: AsyncSyscalls::ReadFile,
    };

    crate::syscall::asynchronous::async_syscall::send_async_syscall(submission_buffer, a);
    AsyncOpenedFile { afd: *afd }
}

pub(in crate::syscall::asynchronous) fn handle_async_read(
    ptr: *const u8,
    len: usize,
    returned_values: &mut AsyncReturnedValues,
) -> usize {
    let syscall_data: &AsyncReadSyscallData = unsafe {
        let slice = core::slice::from_raw_parts(ptr, len);
        crate::utils::struct_to_slice::u8_slice_to_any(slice)
    };

    let fd = match AsyncFileDescriptor::from_usize(syscall_data.afd) {
        AsyncFileDescriptor::FileDescriptor(val) => val,
        AsyncFileDescriptor::AsyncSyscallReturnValue(val) => match returned_values.map.get(&val) {
            None => return ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize,
            Some((syscall_type, returned_value)) => {
                if let AsyncSyscalls::OpenFile = syscall_type {
                    if *returned_value & ONLY_MSB_OF_USIZE > 0 {
                        return *returned_value;
                    } else {
                        *returned_value
                    }
                } else {
                    return ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize;
                }
            }
        },
    };

    handle_read(fd, syscall_data.length, syscall_data.buffer) as usize
}
