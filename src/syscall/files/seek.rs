use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;

const ONLY_MSB_OF_USIZE: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

pub fn seek(fd: usize, position: usize) {
    unsafe {
        syscall2(fd, position, Syscalls::SeekFile as usize);
    }
}

pub fn handle_seek(context: &mut ExceptionContext) {
    let fd = context.gpr[0] as usize;
    let position = context.gpr[1] as usize;

    unsafe {
        let current_task = crate::scheduler::get_current_task_context();
        let fd_table = &mut (*current_task).file_descriptor_table;

        if !fd_table.exists(fd) {
            context.gpr[0] = (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
            return;
        }
        let opened_file = fd_table.get_file_mut(fd).unwrap();
        opened_file.seek(position);
    }
}
