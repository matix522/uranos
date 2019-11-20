use alloc::boxed::Box;

use crate::interupt::ExceptionContext;

/// Stack size of task in bytes
pub const TASK_STACK_SIZE: usize = 0x8000;

extern "C" {
    /// Signal end of scheduling, zero x0 - x18 and jump to x19
    fn new_task_func() -> ();
    fn drop_el0(context: &ExceptionContext) -> !;

}

/// Error regarding tasks
#[derive(Debug)]
pub enum TaskError {
    /// Limit of tasks has been reached when trying to add next task
    TaskLimitReached,
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
#[derive(Debug)]
/// Registers x19-x29, sp, lr
pub struct GPR {
    x19: [u64; 11],
    sp: u64,
    lr: u64,
}

#[repr(C)]
#[derive(Debug)]
/// Structure containing context of a single task
pub struct TaskContext {
    /// General Purpose Registers
    pub(super) gpr: GPR,
    /// State of task
    pub(super) task_state: TaskStates,
    /// Counter meaning how many time quants has remained to this task during current cycle
    pub(super) counter: u32,
    /// Number of time quants given in one round
    pub(super) priority: u32,
    /// Currently unused
    preemption_count: u32,
    /// "Pointer" to kernel space task stack
    stack: Option<Box<[u8]>>,
    /// "Pointer" to user 33554432space task stack
    user_stack: Option<Box<[u8]>>,
    /// Is user task
    has_user_space: bool,
}

impl TaskContext {
    const fn empty() -> Self {
        TaskContext {
            gpr: GPR {
                x19: [0; 11],
                sp: 0,
                lr: 0,
            },
            counter: 0,
            priority: 0,
            preemption_count: 0,
            stack: None,
            user_stack: None,

            task_state: TaskStates::NotStarted,
            has_user_space: false,
        }
    }

    /// create new task
    pub fn new(start_function: extern "C" fn(), priority: u32, is_user_task: bool) -> Self {
        let mut task = Self::empty();
        // Initialize task
        let stack = Box::new([0; TASK_STACK_SIZE]);

        // Initialize priorities
        task.priority = priority;
        task.counter = priority;

        // set lr new_task_func to clear up registers, finalize scheduling and jump to start_function on first run of task
        task.gpr.lr = new_task_func as *const () as u64;

        if is_user_task {
            let user_stack = Box::new([0; TASK_STACK_SIZE]);

            // x19 of task is address of usserspace transition start_function
            task.gpr.x19[0] = switch_to_user_space as *const () as u64;
            // x20 of task is address of user start_function
            task.gpr.x19[1] = start_function as *const () as u64;
            // x21 of task is user stack pointer
            task.gpr.x19[2] =
                unsafe { (*user_stack).as_ptr().add(TASK_STACK_SIZE) as *const () as u64 };

            task.user_stack = Some(user_stack);
            task.has_user_space = true;
        } else {
            // x19 of task is address of start_function
            task.gpr.x19[0] = start_function as *const () as u64;
        }

        unsafe {
            // set stack pointer to the oldest address of task stack space
            task.gpr.sp = (*stack).as_ptr().add(TASK_STACK_SIZE) as *const () as u64;
        }

        task.stack = Some(stack);
        task
    }

    /// Adds task to task vector and set state to running
    pub fn start_task(self) -> Result<(), TaskError> {
        super::SCHEDULER.lock().add_task(self)
    }
}

extern "C" fn switch_to_user_space(start_function: u64, stack_pointer: u64) -> ! {
    let mut context = ExceptionContext {
        gpr: crate::interupt::GPR { x: [0; 31] },
        spsr_el1: 0,
        elr_el1: start_function,
        esr_el1: 0,
        sp_el0: stack_pointer,
    };
    unsafe {
        drop_el0(&context);
    }
}
