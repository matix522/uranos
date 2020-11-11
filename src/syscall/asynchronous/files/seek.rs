use super::*;
use crate::syscall::asynchronous::async_returned_values::*;
use crate::syscall::asynchronous::async_syscall::*;
use crate::syscall::asynchronous::future_async_syscall_result::FutureAsyncSyscallResult;
use crate::utils::circullar_buffer::*;
use crate::vfs;
use alloc::string::String;
use alloc::string::ToString;
use num_traits::FromPrimitive;

pub struct AsyncSeekSyscallData {
    pub afd: usize,
    pub value: isize,
    pub seek_type: usize,
}

impl AsyncSeekSyscallData {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { crate::utils::struct_to_slice::any_as_u8_slice(self) }
    }
}

pub fn seek(
    afd: &AsyncFileDescriptor,
    value: isize,
    seek_type: vfs::SeekType,
    id: usize,
    submission_buffer: &mut CircullarBuffer,
) {
    let data = AsyncSeekSyscallData {
        afd: afd.to_usize(),
        value,
        seek_type: seek_type as usize,
    };

    let bytes = data.as_bytes();

    let a: AsyncSyscall = AsyncSyscall {
        data: bytes,
        id,
        data_size: bytes.len(),
        syscall_type: AsyncSyscalls::SeekFile,
    };

    crate::syscall::asynchronous::async_syscall::send_async_syscall(submission_buffer, a);
}

pub fn handle_async_seek(
    ptr: *const u8,
    len: usize,
    returned_values: &mut AsyncReturnedValues,
) -> usize {
    let syscall_data: &AsyncSeekSyscallData = unsafe {
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

    let current_task = crate::scheduler::get_current_task_context();
    let fd_table = unsafe { &mut (*current_task).file_descriptor_table };

    if !fd_table.exists(fd) {
        return ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize;
    }
    let opened_file = fd_table.get_file_mut(fd).unwrap();
    let seek_type = vfs::SeekType::from_usize(syscall_data.seek_type)
        .unwrap_or_else(|| panic!("Wrong type of SeekType sent: {}", syscall_data.seek_type));
    match vfs::seek(opened_file, syscall_data.value, seek_type) {
        Ok(val) => val,
        Err(err) => ONLY_MSB_OF_USIZE | err as usize,
    }
}
