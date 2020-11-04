use crate::interupts::ExceptionContext;

pub fn handle_get_read_buffer(context: &mut ExceptionContext) {
    unsafe {
        context.gpr[0] =
            &(*crate::scheduler::get_current_task_context()).read_buffer as *const _ as u64;
    }
}
