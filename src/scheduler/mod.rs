#![deny(missing_docs)]
#![deny(warnings)]

use crate::sync::nulllock::NullLock;
use alloc::vec::Vec;

/// Module containing definition of tasks.
pub mod task_context;
pub use task_context::*;

extern "C" {
    /// Change CPU context from prev task to next task
    fn cpu_switch_to(prev_task_addr: u64, next_task_addr: u64) -> ();
    /// Change CPU context to init task (dummy lands in unused x0 for sake of simplicity)
    fn cpu_switch_to_first(dummy: u64, init_task_addr: u64) -> !;

}

/// Main System Scheduler
pub static SCHEDULER: NullLock<Scheduler> = NullLock::new(Scheduler::new());

/// Maximal number of scheduled tasks
pub const MAX_TASK_COUNT: usize = 16;

/// Signal end of scheduling
#[no_mangle]
pub extern "C" fn schedule_tail() {
    crate::interupt::handlers::end_scheduling();
}

/// Round-robin with priority scheduling algorithm choosing next task and switching to it
pub fn schedule() -> Result<usize, TaskError> {
    SCHEDULER.lock().schedule()
}
/// Function statring scheduling process, should not return
pub fn start() -> Result<!, TaskError> {
    SCHEDULER.lock().start()
}

/// Definition od System Scheduler
pub struct Scheduler {
    tasks: Vec<TaskContext>,
    current_running_task: usize,
}

impl Scheduler {
    // Creates Scheduler
    const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_running_task: 0,
        }
    }
    /// Round-robin with priority scheduling algorithm choosing next task and switching to it
    pub fn schedule(&mut self) -> Result<usize, TaskError> {
        let next_task_pid;
        let tasks = &mut self.tasks;

        'find_task: loop {
            for (i, task) in tasks.iter_mut().enumerate() {
                // get mutable reference for currently examined task

                match task.task_state {
                    // if curr_task is in state that it should be scheduled and task has unused quant of time
                    TaskStates::Running | TaskStates::NotStarted if task.counter > 0 => {
                        // decrease counter and mark it as next task
                        task.counter -= 1;
                        next_task_pid = i;
                        break 'find_task;
                    }
                    // in other states ignore this task
                    _ => {}
                }
            }
            for task in tasks.iter_mut() {
                task.counter = task.priority;
            }
        }
        self.change_task(next_task_pid)?;

        Ok(next_task_pid)
    }
    /// Function statring scheduling process, should not return
    pub fn start(&mut self) -> Result<!, TaskError> {
        let tasks = &mut self.tasks;

        if tasks.len() == 0 {
            return Err(TaskError::ChangeTaskError);
        }

        let mut init_task = &mut tasks[0];
        init_task.counter = init_task.priority - 1;

        let init_task_addr = init_task as *mut TaskContext as u64;

        unsafe {
            cpu_switch_to_first(0, init_task_addr);
        }
    }
    /// Function that changes current tasks and stores context of previous one in his TaskContext structure
    fn change_task(&mut self, next_task: usize) -> Result<(), TaskError> {
        let tasks = &mut self.tasks;

        if next_task >= tasks.len() {
            return Err(TaskError::InvalidTaskReference);
        }

        let prev_task_addr = &tasks[self.current_running_task] as *const TaskContext as u64;
        let next_task_addr = &tasks[next_task] as *const TaskContext as u64;

        self.current_running_task = next_task;

        unsafe {
            cpu_switch_to(prev_task_addr, next_task_addr);
        }

        Ok(())
    }

    /// Submit task for scheduling
    fn submit_task(&mut self, task_context: TaskContext) -> Result<(), TaskError> {
        if self.tasks.len() >= MAX_TASK_COUNT {
            return Err(TaskError::TaskLimitReached);
        }
        self.tasks.push(task_context);

        Ok(())
    }
}

global_asm!(include_str!("change_task.S"));
