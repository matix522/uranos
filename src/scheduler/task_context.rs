use super::task_stack;
use crate::interupts::ExceptionContext;

/// Stack size of task in bytes
pub const TASK_STACK_SIZE: usize = 0x8000;

/// Error regarding tasks
#[derive(Debug)]
pub enum TaskError {
    /// Limit of tasks has been reached when trying to add next task
    TaskLimitReached,
    /// Stack could not be allocated
    StackAllocationFail,
    /// Referenced to task out of bounds of array
    InvalidTaskReference,
    /// Error in changing task
    ChangeTaskError,
}

#[repr(C)]
#[derive(Debug)]
/// States that task can be in
pub enum TaskStates {
    /// Task is created, by not started
    NotStarted = 0,
    /// Task is running and managed by scheduler
    Running = 1,
    /// Task is suspended and skipped by scheduler
    Suspended = 2,
    /// Task is dead and waiting to clean after it
    Dead = 3,
}

#[repr(C)]
pub struct TaskContext {
    pub(super) exception_context: *mut ExceptionContext,
    pub(super) state: TaskStates,
    stack: Option<task_stack::TaskStack>,
}

// ONLY TEMPORARY SOLUTION
unsafe impl Sync for TaskContext {}
unsafe impl Send for TaskContext {}

impl TaskContext {
    const fn empty() -> Self {
        TaskContext {
            exception_context: core::ptr::null_mut(),
            state: TaskStates::NotStarted,
            stack: None,
        }
    }

    pub fn new(start_function: extern "C" fn(), is_kernel: bool) -> Result<Self, TaskError> {
        let mut task: TaskContext = Self::empty();
        let mut exception_context = ExceptionContext {
            gpr: [0; 30],
            lr: 0,
            elr_el1: 0,
            spsr_el1: 0,
            esr_el1: 0,
            far_el1: 0,
            sp: 0,
        };
        let user_address = |address: usize| (address & !crate::KERNEL_OFFSET) as u64;

        exception_context.spsr_el1 = if is_kernel { 0b0101 } else { 0b0000 };

        let stack =
            task_stack::TaskStack::new(TASK_STACK_SIZE).ok_or(TaskError::StackAllocationFail)?;

        let exception_context_ptr =
            (stack.base() - core::mem::size_of::<ExceptionContext>()) as *mut ExceptionContext;

        task.stack = Some(stack);
        if is_kernel {
            exception_context.elr_el1 = start_function as *const () as u64;
            exception_context.sp = exception_context_ptr as u64;
            task.exception_context = exception_context_ptr;
        } else {
            exception_context.elr_el1 = user_address(start_function as *const () as usize);
            exception_context.sp = user_address(exception_context_ptr as usize);
            task.exception_context = user_address(exception_context_ptr as usize) as *mut _;
        }

        // # Safety: exception_context is stack variable and exception_context_ptr is valid empty space for this data.
        unsafe {
            core::ptr::copy_nonoverlapping(
                &exception_context as *const _,
                exception_context_ptr,
                1,
            );
        }
        Ok(task)
    }
}
