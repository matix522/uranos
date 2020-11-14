use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;
use core::slice;
use num_traits::FromPrimitive;

use crate::utils::ONLY_MSB_OF_USIZE;

pub fn write(fd: usize, bytes: &[u8]) -> Result<(), vfs::FileError> {
    let val: usize;

    unsafe {
        val = syscall3(
            fd,
            bytes.as_ptr() as usize,
            bytes.len(),
            Syscalls::WriteFile as usize,
        );
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
        Ok(())
    }
}

pub fn handle_write(context: &mut ExceptionContext) {
    let fd = context.gpr[0] as usize;
    let ptr = context.gpr[1] as *const u8;
    let len = context.gpr[2] as usize;

    let data = unsafe { slice::from_raw_parts(ptr, len) };

    let current_task = crate::scheduler::get_current_task_context();
    let fd_table = unsafe { &mut (*current_task).file_descriptor_table };
    let opened_file = fd_table.get_file_mut(fd).unwrap();
    match vfs::write(opened_file, data) {
        Ok(_) => {
            context.gpr[0] = 0;
        }
        Err(err) => {
            context.gpr[0] = (ONLY_MSB_OF_USIZE | err as usize) as u64;
        }
    }
}
