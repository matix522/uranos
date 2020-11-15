use super::PIPE_QUEUE_GRANULATION;
use crate::interupts::ExceptionContext;
use crate::scheduler;
use crate::syscall::*;
use crate::utils::ONLY_MSB_OF_USIZE;
use crate::vfs;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use num_traits::FromPrimitive;
use crate::scheduler::task_context::*;

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
    let mut buffer = context.gpr[2] as *mut u8;
    /// Special file descriptors:
    /// 0: STDIN (UART)
    /// 1: STDOUT (UART)
    /// 2: PIPEIN
    /// 3: PIPEOUT
    match fd {
        0 => {
            panic!("Not implemented yet");
        }
        1 => {
            context.gpr[0] =
                (ONLY_MSB_OF_USIZE | vfs::FileError::CannotReadWriteOnlyFile as usize) as u64
        }
        2 => {
            let current_task = crate::scheduler::get_current_task_context();
            if let Some(pid) = unsafe { (*current_task).pipe_from } {
                if let Ok(task_ptr) = scheduler::get_task_context(pid) {
                    let task: &mut TaskContext = unsafe { &mut (*task_ptr) };
                    if let TaskStates::Dead = *task.get_state(){
                        context.gpr[0] = (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
                        return;
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
                                context.gpr[0] = (length - data_to_go) as u64;
                                return;
                            }
                        } else {
                            context.gpr[0] = length as u64;
                            return;
                        }
                    }
                }
            } else {
                context.gpr[0] =
                    (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64
            }
        }
        3 => {
            context.gpr[0] =
                (ONLY_MSB_OF_USIZE | vfs::FileError::CannotReadWriteOnlyFile as usize) as u64
        }
        _ => {
            let current_task = crate::scheduler::get_current_task_context();
            let fd_table = unsafe { &mut (*current_task).file_descriptor_table };

            if !fd_table.exists(fd) {
                context.gpr[0] =
                    (ONLY_MSB_OF_USIZE | vfs::FileError::ReadOnClosedFile as usize) as u64;
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
    };
}
