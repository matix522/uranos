use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;
use core::slice;
use num_traits::FromPrimitive;

use super::PIPE_QUEUE_GRANULATION;
use crate::scheduler::task_context::*;
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

pub fn pipe_write_handler(data: &[u8]) -> u64 {
    let current_task = crate::scheduler::get_current_task_context();
    let current_task: &mut TaskContext = unsafe { &mut (*current_task) };
    let mut data_to_go = data.len();
    let mut data_sent = 0;
    loop {
        if data_to_go > 0 {
            if data_to_go >= PIPE_QUEUE_GRANULATION {
                current_task.push_back_item_to_pipe_queue(
                    data[data_sent..(data_sent + PIPE_QUEUE_GRANULATION)].to_vec(),
                );
                data_sent += PIPE_QUEUE_GRANULATION;
                data_to_go -= PIPE_QUEUE_GRANULATION;
            } else {
                current_task.push_back_item_to_pipe_queue(data[data_sent..].to_vec());
                return 0;
            }
        } else {
            return 0;
        }
    }
}

pub fn vfs_write_handler(data: &[u8], fd: usize) -> u64 {
    let current_task = crate::scheduler::get_current_task_context();
    let fd_table = unsafe { &mut (*current_task).file_descriptor_table };
    let opened_file = fd_table.get_file_mut(fd).unwrap();
    match vfs::write(opened_file, data) {
        Ok(_) => 0,
        Err(err) => (ONLY_MSB_OF_USIZE | err as usize) as u64,
    }
}

pub fn handle_write(context: &mut ExceptionContext) {
    let fd = context.gpr[0] as usize;
    let ptr = context.gpr[1] as *const u8;
    let len = context.gpr[2] as usize;

    let data = unsafe { slice::from_raw_parts(ptr, len) };

    // Special file descriptors:
    // 0: STDIN (UART)
    // 1: STDOUT (UART)
    // 2: PIPEIN
    // 3: PIPEOUT
    context.gpr[0] = match fd {
        0 => (ONLY_MSB_OF_USIZE | vfs::FileError::ModifyingWithoutWritePermission as usize) as u64,
        1 => {
            let string = unsafe { core::str::from_utf8_unchecked(data) };
            crate::print!("{}", string);
            0
        }
        2 => (ONLY_MSB_OF_USIZE | vfs::FileError::ModifyingWithoutWritePermission as usize) as u64,
        3 => pipe_write_handler(data),
        _ => vfs_write_handler(data, fd),
    };
}
