use crate::interupts::ExceptionContext;
use super::task_stack;

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
pub struct TaskContext{
    pub(super) exception_context: ExceptionContext,
    pub(super) state: TaskStates,
    pub(super) priority: u32,
    pub(super) counter: u32,
    stack: Option<task_stack::TaskStack>,
}

impl TaskContext{
    const fn empty() -> Self{
        TaskContext{
            exception_context: ExceptionContext{
                gpr: [0; 30],
                lr: 0,
                elr_el1: 0,
                spsr_el1: 0,
                esr_el1: 0,
                far_el1: 0,
            },
            state: TaskStates::NotStarted,
            priority: 0,
            counter: 0,
            stack: None,
        }
    }

    pub fn new(
        start_function: extern "C" fn(),
        priority: u32,
    ) -> Result<Self, TaskError>{
        let mut task: TaskContext = Self::empty();

        let stack = task_stack::TaskStack::new(TASK_STACK_SIZE).ok_or(TaskError::StackAllocationFail)?;
        
        task.stack = Some(stack);
        task.priority = priority;
        
        
        task.exception_context.elr_el1 = start_function as *const () as u64;

        Ok(task)
    }
}