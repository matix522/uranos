use super::*;
use crate::syscall::asynchronous::async_returned_values::*;
use crate::syscall::asynchronous::async_syscall::*;
use crate::utils::circullar_buffer::*;
use crate::vfs;

pub struct AsyncCloseSyscallData {
    pub afd: usize,
}

impl AsyncCloseSyscallData {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { crate::utils::struct_to_slice::any_as_u8_slice(self) }
    }
}

pub fn close(afd: &AsyncFileDescriptor, id: usize, submission_buffer: &mut CircullarBuffer) {
    let data = AsyncCloseSyscallData {
        afd: afd.to_usize(),
    };

    let bytes = data.as_bytes();

    let a: AsyncSyscall = AsyncSyscall {
        data: bytes,
        id,
        data_size: bytes.len(),
        syscall_type: AsyncSyscalls::CloseFile,
    };

    crate::syscall::asynchronous::async_syscall::send_async_syscall(submission_buffer, a);
}

pub fn handle_async_close(
    ptr: *const u8,
    len: usize,
    returned_values: &mut AsyncReturnedValues,
) -> usize {
    let syscall_data: &AsyncCloseSyscallData = unsafe {
        let slice = core::slice::from_raw_parts(ptr, len);
        crate::utils::struct_to_slice::u8_slice_to_any(slice)
    };

    let fd = match AsyncFileDescriptor::from_usize(syscall_data.afd) {
        AsyncFileDescriptor::FileDescriptor(val) => val,
        AsyncFileDescriptor::AsyncSyscallReturnValue(val) => match returned_values.map.get(&val) {
            None => return ONLY_MSB_OF_USIZE | vfs::FileError::AttemptToCloseClosedFile as usize,
            Some((syscall_type, returned_value)) => {
                if let AsyncSyscalls::OpenFile = syscall_type {
                    if *returned_value & ONLY_MSB_OF_USIZE > 0 {
                        return *returned_value;
                    } else {
                        *returned_value
                    }
                } else {
                    return ONLY_MSB_OF_USIZE | vfs::FileError::AttemptToCloseClosedFile as usize;
                }
            }
        },
    };

    let current_task = crate::scheduler::get_current_task_context();
    let fd_table = unsafe { &mut (*current_task).file_descriptor_table };

    if !fd_table.exists(fd) {
        return ONLY_MSB_OF_USIZE | vfs::FileError::AttemptToCloseClosedFile as usize;
    }

    let mut opened_file = fd_table.delete_file(fd).unwrap();
    let ret = vfs::close(&mut opened_file);
    if ret.is_err() {
        return ONLY_MSB_OF_USIZE | ret.err().unwrap() as usize;
    }
    0
}
