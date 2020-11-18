use super::resolve_fd;
use crate::interupts::ExceptionContext;
use crate::scheduler;
use crate::scheduler::task_context::*;
use crate::syscall::*;
use crate::utils::ONLY_MSB_OF_USIZE;
use crate::vfs;
use num_traits::FromPrimitive;

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

pub fn read_from_pipe_handler(length: usize, mut buffer: *mut u8) -> u64 {
    let current_task = crate::scheduler::get_current_task_context();
    if let Some(pid) = unsafe { (*current_task).pipe_from } {
        if let Ok(task_ptr) = scheduler::get_task_context(pid) {
            let task: &mut TaskContext = unsafe { &mut (*task_ptr) };
            if let TaskStates::Dead = *task.get_state() {
                return (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
            }
            if let TaskStates::Zombie = *task.get_state() {
                if task.is_pipe_queue_empty() {
                    return (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
                }
            }
            let mut data_to_go = length;
            loop {
                if data_to_go > 0 {
                    if let Some(element) = task.get_item_from_pipe_queue() {
                        if element.len() <= data_to_go {
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    &element[..] as *const [u8] as *const u8,
                                    buffer,
                                    element.len(),
                                );
                                buffer = buffer.add(element.len());
                                data_to_go -= element.len()
                            }
                        } else {
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    &element[..] as *const [u8] as *const u8,
                                    buffer,
                                    data_to_go,
                                );
                                task.push_front_item_to_pipe_queue(element[data_to_go..].to_vec());
                            }
                        }
                    } else {
                        return (length - data_to_go) as u64;
                    }
                } else {
                    return length as u64;
                }
            }
        }
    }
    return (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
}

pub fn read_from_vfs_handler(fd: usize, length: usize, buffer: *mut u8) -> u64 {
    let current_task = crate::scheduler::get_current_task_context();
    let fd_table = unsafe { &mut (*current_task).file_descriptor_table };

    if !fd_table.exists(fd) {
        return (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
    }
    let opened_file = fd_table.get_file_mut(fd).unwrap();
    match vfs::read(opened_file, length) {
        Ok(data) => {
            unsafe {
                core::ptr::copy_nonoverlapping(data.data, buffer, data.len);
            }
            return data.len as u64;
        }
        Err(err) => {
            return (ONLY_MSB_OF_USIZE | err as usize) as u64;
        }
    }
}

pub fn read_from_stdin_handler(length: usize, buffer: *mut u8) -> u64 {
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, length) };
    let mut stdin = crate::io::INPUT_BUFFER.lock();

    let size = core::cmp::min(buffer.len(), stdin.len());
    for (i, byte) in stdin.iter().take(size).enumerate() {
        buffer[i] = *byte;
    }
    stdin.drain(..size);
    size as u64
}

pub fn handle_read_syscall(context: &mut ExceptionContext) {
    let fd = resolve_fd(context.gpr[0] as usize);
    let length = context.gpr[1] as usize;
    let mut buffer = context.gpr[2] as *mut u8;
    // Special file descriptors:
    // 0: STDIN (UART)
    // 1: STDOUT (UART)
    // 2: PIPEIN
    // 3: PIPEOUT
    context.gpr[0] = handle_read(fd, length, buffer);
}

pub fn handle_read(fd: usize, length: usize, buffer: *mut u8) -> u64 {
    // Special file descriptors:
    // 0: STDIN (UART)
    // 1: STDOUT (UART)
    // 2: PIPEIN
    // 3: PIPEOUT
    match resolve_fd(fd) {
        0 => read_from_stdin_handler(length, buffer) as u64,
        1 => (ONLY_MSB_OF_USIZE | vfs::FileError::CannotReadWriteOnlyFile as usize) as u64,
        2 => read_from_pipe_handler(length, buffer),
        3 => (ONLY_MSB_OF_USIZE | vfs::FileError::CannotReadWriteOnlyFile as usize) as u64,
        _ => read_from_vfs_handler(fd, length, buffer),
    }
}
