use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;
use num_traits::FromPrimitive;

use crate::utils::ONLY_MSB_OF_USIZE;

pub fn close(fd: usize) -> Result<(), vfs::FileError> {
    let val: usize;
    unsafe {
        val = syscall1(fd, Syscalls::CloseFile as usize);
    }
    if val & ONLY_MSB_OF_USIZE > 0 {
        Err(
            vfs::FileError::from_usize(val & !ONLY_MSB_OF_USIZE).unwrap_or_else(|| {
                panic!(
                    "Unknown error during file closing: {}",
                    val & !ONLY_MSB_OF_USIZE
                )
            }),
        )
    } else {
        Ok(())
    }
}

pub fn handle_close(context: &mut ExceptionContext) {
    let fd = context.gpr[0] as usize;

    unsafe {
        let current_task = crate::scheduler::get_current_task_context();
        let fd_table = &mut (*current_task).file_descriptor_table;

        if !fd_table.exists(fd) {
            context.gpr[0] =
                (ONLY_MSB_OF_USIZE | vfs::FileError::AttemptToCloseClosedFile as usize) as u64;
            return;
        }

        let mut opened_file = fd_table.delete_file(fd).unwrap();
        let ret = vfs::close(&mut opened_file);
        if ret.is_err() {
            context.gpr[0] = (ONLY_MSB_OF_USIZE | ret.err().unwrap() as usize) as u64;
            return;
        }
        context.gpr[0] = 0;
    }
}
