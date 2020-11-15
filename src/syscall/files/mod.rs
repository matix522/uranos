pub mod close;
pub mod create;
pub mod delete;
pub mod file_descriptor_map;
pub mod open;
pub mod read;
pub mod seek;
pub mod write;

pub const PIPE_QUEUE_GRANULATION: usize = 64;

pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const PIPEIN: usize = 2;
pub const PIPEOUT: usize = 3;

use crate::interupts::ExceptionContext;
use crate::scheduler;

pub fn handle_set_pipe_read_on_pid(e: &mut ExceptionContext) {
    let pid = e.gpr[0] as usize;
    let current_task = scheduler::get_current_task_context();
    unsafe {
        (*current_task).pipe_from = Some(pid);
    }
}
