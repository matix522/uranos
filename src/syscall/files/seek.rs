use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;
use num_traits::FromPrimitive;

use crate::utils::ONLY_MSB_OF_USIZE;

pub fn seek(fd: usize, value: isize, seek_type: vfs::SeekType) -> Result<usize, vfs::FileError> {
    let val: usize;
    unsafe {
        val = syscall3(
            fd,
            value as usize,
            seek_type as usize,
            Syscalls::SeekFile as usize,
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
        Ok(val)
    }
}

pub fn handle_seek(context: &mut ExceptionContext) {
    let fd = context.gpr[0] as usize;
    let difference = context.gpr[1] as isize;
    let seek_type = vfs::SeekType::from_u64(context.gpr[2])
        .unwrap_or_else(|| panic!("Wrong type of SeekType sent: {}", context.gpr[2]));

    if fd < 4 {
        context.gpr[0] = (ONLY_MSB_OF_USIZE | vfs::FileError::CannotSeekSpecialFile as usize) as u64;
        return;
    }

    let current_task = crate::scheduler::get_current_task_context();
    let fd_table = unsafe { &mut (*current_task).file_descriptor_table };

    if !fd_table.exists(fd) {
        context.gpr[0] = (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
        return;
    }
    let opened_file = fd_table.get_file_mut(fd).unwrap();
    match vfs::seek(opened_file, difference, seek_type) {
        Ok(val) => {
            context.gpr[0] = val as u64;
        }
        Err(err) => {
            context.gpr[0] = (ONLY_MSB_OF_USIZE | err as usize) as u64;
        }
    }
}
