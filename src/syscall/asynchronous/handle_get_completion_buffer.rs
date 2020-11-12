use crate::interupts::ExceptionContext;

pub fn handle_get_completion_buffer(context: &mut ExceptionContext) {
    unsafe {
        context.gpr[0] =
            &(*crate::scheduler::get_current_task_context()).completion_buffer as *const _ as u64;
    }
}
