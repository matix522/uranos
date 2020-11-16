use super::*;
use crate::syscall::asynchronous::async_returned_values::*;
use crate::syscall::asynchronous::async_syscall::*;
use crate::syscall::files::seek::vfs_seek_handler;
use crate::utils::circullar_buffer::*;
use crate::vfs;
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
) -> AsyncOpenedFile {
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
    AsyncOpenedFile { afd: *afd }
}

pub(in crate::syscall::asynchronous) fn handle_async_seek(
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

    if fd < 4 {
        return ONLY_MSB_OF_USIZE | vfs::FileError::CannotSeekSpecialFile as usize;
    }

    let seek_type = vfs::SeekType::from_usize(syscall_data.seek_type)
        .unwrap_or_else(|| panic!("Wrong type of SeekType sent: {}", syscall_data.seek_type));

    vfs_seek_handler(fd, syscall_data.value, seek_type) as usize
}
