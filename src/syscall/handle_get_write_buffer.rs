use super::*;
use crate::interupts::ExceptionContext;
use crate::scheduler::task_context::*;

pub fn handle_get_write_buffer(context: &mut ExceptionContext) -> &mut ExceptionContext {
    unsafe{
        context.gpr[0] = &(*crate::scheduler::get_current_task_context()).write_buffer as *const _ as u64;
    }
    context
}