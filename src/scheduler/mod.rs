// #![deny(missing_docs)]
// #![deny(warnings)]

use crate::sync::nulllock::NullLock;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub mod init;

extern "C" {
    /// Change CPU context from prev task to next task
    fn cpu_switch_to(prev_task_addr: u64, next_task_addr: u64) -> ();
    /// Change CPU context to init task (dummy lands in unused x0 for sake of simplicity)
    fn cpu_switch_to_first(dummy: u64, init_task_addr: u64) -> ();
    /// Signal end of scheduling, zero x0 - x18 and jump to x19
    fn new_task_func() -> ();
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
    gpr: GPR,
    /// State of task
    task_state: TaskStates,
    /// Counter meaning how many time quants has remained to this task during current cycle
    counter: u32,
    /// Number of time quants given in one round
    priority: u32,
    /// Currently unused
    preemption_count: u32,
    /// "Pointer" to kernel space task stack
    stack: Option<Box<[u8]>>,
    /// "Pointer" to user 33554432space task stack
    user_stack: Option<Box<[u8]>>,
    /// Is user task
    has_user_space: bool,
}

/// Stack size of task in bytes
const TASK_STACK_SIZE: usize = 0x8000;

/// Vector of tasks
pub static TASKS: NullLock<Vec<TaskContext>> = NullLock::new(Vec::new());

/// Maximal number of scheduled tasks
pub const MAX_TASK_COUNT: usize = 16;

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
        let mut tasks = TASKS.lock();

        if tasks.len() >= MAX_TASK_COUNT {
            return Err(TaskError::TaskLimitReached);
        }
        tasks.push(self);

        Ok(())
    }
}
use crate::interupt::*;
extern "C" {
    pub fn drop_el0(context: &ExceptionContext);
}

pub extern "C" fn switch_to_user_space(start_function: u64, stack_pointer: u64) -> ! {
    let mut context = ExceptionContext {
        gpr: crate::interupt::GPR { x: [0; 31] },
        spsr_el1: 0,
        elr_el1: start_function,
        esr_el1: 0,
        sp_el0: stack_pointer,
    };
    crate::println!("{:?}", context);
    crate::println!("{}", init::init as *const () as u64);
    unsafe {
        drop_el0(&context);
    }
    loop {}
}

/// Signal end of scheduling
#[no_mangle]
pub extern "C" fn schedule_tail() {
    crate::interupt::handlers::end_scheduling();
}

/// Round-robin with priority scheduling algorithm choosing next task and switching to it
pub fn schedule() -> () {
    let mut next_task_found: bool = false;
    let mut nothing_found: bool = false;
    let mut next_task_pid: usize = 0;

    let mut tasks = TASKS.lock();

    while !next_task_found {
        for i in 0..tasks.len() {
            // get mutable reference for currently examined task
            let curr_task: &mut TaskContext = &mut tasks[i];
            match curr_task.task_state {
                // if curr_task is in state that it should be scheduled
                TaskStates::Running | TaskStates::NotStarted => {
                    // if our task has unused quant of time decrease counter and mark it as next task
                    if curr_task.counter > 0 {
                        curr_task.counter -= 1;
                        next_task_pid = i;
                        next_task_found = true;
                        break;
                    }
                }
                // in other states ignore this task
                _ => {
                    continue;
                }
            }
        }

        if next_task_found {
            break;
        }

        if !nothing_found {
            nothing_found = true;
        } else {
            for i in 0..tasks.len() {
                tasks[i].counter = tasks[i].priority;
            }
            nothing_found = false;
        }
    }
    unsafe {
        match change_task(next_task_pid) {
            Ok(_) => {}
            Err(_) => aarch64::halt(),
        };
    }
}

/// Function statring scheduling process
pub fn start_scheduling(init_fun: extern "C" fn()) -> Result<!, TaskError> {
    let mut tasks = TASKS.lock();
    if tasks.len() == 0 {
        return Err(TaskError::ChangeTaskError);
    }
    unsafe {
        let mut init_task = &mut tasks[0];
        init_task.counter = 0;

        let init_task_addr = init_task as *const TaskContext as u64;

        cpu_switch_to_first(0, init_task_addr);
    }
    // should not ever be here
    loop {}
}

static mut PREVIOUS_TASK_PID: usize = 0;

pub fn get_current_task_PID() -> usize{
    unsafe{PREVIOUS_TASK_PID}
}

pub fn get_current_task_priority() -> u32{
    let tasks = TASKS.lock();
    unsafe{tasks[PREVIOUS_TASK_PID].priority}
}

/// Function that changes current tasks and stores context of previous one in his TaskContext structure
pub unsafe fn change_task(next: usize) -> Result<(), TaskError> {
    let tasks = TASKS.lock();

    if PREVIOUS_TASK_PID >= tasks.len() || next >= tasks.len() {
        return Err(TaskError::InvalidTaskReference);
    }

    let prev_task_addr = &tasks[PREVIOUS_TASK_PID] as *const TaskContext as u64;
    let next_task_addr = &tasks[next] as *const TaskContext as u64;

    PREVIOUS_TASK_PID = next;
    cpu_switch_to(prev_task_addr, next_task_addr);

    Ok(())
}

global_asm!(include_str!("change_task.S"));
