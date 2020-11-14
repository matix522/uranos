use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;
use num_traits::FromPrimitive;

use crate::utils::ONLY_MSB_OF_USIZE;

pub fn read(fd: usize, length: usize, buffer: *mut u8) -> Result<usize, vfs::FileError> {
    let val: usize;
    unsafe {
        val = syscall3(fd, length, buffer as usize, Syscalls::ReadFile as usize);
    }
    if val & ONLY_MSB_OF_USIZE > 0 {
        Err(
            vfs::FileError::from_usize(val & !ONLY_MSB_OF_USIZE).unwrap_or_else(|| {
                panic!(
                    "Unknown error during file opening: {}",
                    val & !ONLY_MSB_OF_USIZE
                )
            }),
        )
    } else {
        Ok(val)
    }
}

pub fn handle_read(context: &mut ExceptionContext) {
    let fd = context.gpr[0] as usize;
    let length = context.gpr[1] as usize;
    let buffer = context.gpr[2] as *mut u8;

    let current_task = crate::scheduler::get_current_task_context();
    let fd_table = unsafe { &mut (*current_task).file_descriptor_table };

    if !fd_table.exists(fd) {
        context.gpr[0] = (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
        return;
    }
    let opened_file = fd_table.get_file_mut(fd).unwrap();
    match vfs::read(opened_file, length) {
        Ok(data) => {
            unsafe {
                core::ptr::copy_nonoverlapping(data.data, buffer, data.len);
            }
            context.gpr[0] = data.len as u64;
        }
        Err(err) => {
            context.gpr[0] = (ONLY_MSB_OF_USIZE | err as usize) as u64;
        }
    }
}
