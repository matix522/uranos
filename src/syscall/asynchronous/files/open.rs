use super::*;
use crate::syscall::asynchronous::async_syscall::*;
use crate::utils::circullar_buffer::*;
use crate::vfs;

pub struct AsyncOpenSyscallData {
    pub filename: &'static str,
    pub with_write: bool,
}

impl AsyncOpenSyscallData {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { crate::utils::struct_to_slice::any_as_u8_slice(self) }
    }
}

pub fn open(
    filename: &'static str,
    with_write: bool,
    id: usize,
    submission_buffer: &mut CircullarBuffer,
) -> AsyncOpenedFile {
    let data = AsyncOpenSyscallData {
        filename,
        with_write,
    };

    let bytes = data.as_bytes();

    let a: AsyncSyscall = AsyncSyscall {
        data: bytes,
        id,
        data_size: bytes.len(),
        syscall_type: AsyncSyscalls::OpenFile,
    };

    crate::syscall::asynchronous::async_syscall::send_async_syscall(submission_buffer, a);
    AsyncOpenedFile {
        afd: AsyncFileDescriptor::AsyncSyscallReturnValue(id),
    }
}

pub fn handle_async_open(ptr: *const u8, len: usize) -> usize {
    let data: &AsyncOpenSyscallData = unsafe {
        let slice = core::slice::from_raw_parts(ptr, len);
        crate::utils::struct_to_slice::u8_slice_to_any(slice)
    };

    let opened_file_res = vfs::open(data.filename, data.with_write);

    let current_task = crate::scheduler::get_current_task_context();

    match opened_file_res {
        Err(e) => {
            crate::println!("DUPAAA {}, {:?}", data.filename, e);
            super::ONLY_MSB_OF_USIZE | (e as usize)
        }

        Ok(opened_file) => unsafe { (*current_task).file_descriptor_table.add_file(opened_file) },
    }
}
