
#![deny(missing_docs)]
// #![deny(warnings)]


use alloc::boxed::Box;
use alloc::vec::Vec;

/// Error regarding tasks
#[derive(Debug)]
pub enum TaskError {
    /// Limit of tasks has been reached when trying to add next task
    TaskLimitReached,
}

#[repr(C)]
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
/// Registers x19-x29, sp, lr
pub struct GPR {
    x: [u64; 11],
    sp: u64,
    lr: u64,
}

#[repr(C)]
/// Structure containing context of a single task
pub struct TaskContext {
    /// General Purpose Registers
    gpr: GPR,
    task_state: TaskStates,
    priority: u32,
    preemption_count: u32,
    stack: Option<Box<[u8]>>,
}

const TASK_STACK_SIZE: usize = 0x80000;
/// Vector of tasks
static mut TASKS: Vec<TaskContext> = Vec::new();

/// Maximal number of scheduled tasks
pub const MAX_TASK_COUNT: usize = 16;

impl TaskContext {
    const fn empty() -> Self {
        TaskContext {
            gpr: GPR {
                x: [0; 11],
                sp: 0,
                lr: 0,
            },
            task_state: TaskStates::NotStarted,
            priority: 0,
            preemption_count: 0,
            stack: None,
        }
    }

    fn new(start_function: fn(), priority: u32) -> Self {
        let mut task = Self::empty();
        let stack = Box::new([0; TASK_STACK_SIZE]);
        task.priority = priority;
        task.gpr.lr = start_function as *const () as u64;
        task.gpr.sp = (*stack).as_ptr() as *const () as u64;
        task.stack = Some(stack);
        task
    }

    fn start(mut self) -> Result<(), TaskError> {
        self.task_state = TaskStates::Running;
        unsafe {
            if TASKS.len() >= MAX_TASK_COUNT {
                return Err(TaskError::TaskLimitReached);
            }
            TASKS.push(self);
        }
        Ok(())
    }
}

// #[link_section = ".task"]
// static TASKS: [TaskContext; MAX_TASK_COUNT] = new_task_table(); //Default::default();// = [TaskContext::new(); MAX_TASK_COUNT];

// #[link_section = ".task.stack"]
// static TASK_STACKS: [TaskStack; MAX_TASK_COUNT] = [TaskStack::new(); MAX_TASK_COUNT] ;

global_asm!(include_str!("change_task.S"));
