pub mod special_return_vals;
pub mod task_context;
pub mod task_memory_manager;
pub mod task_stack;
use crate::device_driver;
use crate::interupts::ExceptionContext;
use alloc::vec::Vec;
use core::time::Duration;
use task_context::*;

pub const MAX_TASK_COUNT: usize = 2048;

extern "C" {
    /// Change CPU context from prev task to next task
    fn cpu_switch_to(
        prev_task_addr: u64,
        next_task_addr: u64,
        prev_table_ptr: u64,
        next_table_ptr: u64,
    );
    /// Change CPU context to init task (dummy lands in unused x0 for sake of simplicity)
    fn cpu_switch_to_first(init_task_addr: u64, init_table_ptr: u64) -> !;

}

device_driver!(
    unsynchronized TASK_MANAGER: TaskManager = TaskManager::new(Duration::from_millis(100))
);

pub fn add_task(task: TaskContext) -> Result<u64, TaskError> {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.add_task(task)
}

pub fn switch_task() {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.switch_task();
}

pub fn start() {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.start();
}

pub fn get_current_task_context() -> *mut TaskContext {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.get_current_task() as *mut TaskContext
}

pub fn get_time_quant() -> Duration {
    let scheduler = TASK_MANAGER.lock();
    scheduler.time_quant
}

pub fn finish_current_task(return_value: u32) {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.finish_current_task(return_value);
}

pub fn get_current_task_pid() -> usize {
    let scheduler = TASK_MANAGER.lock();
    scheduler.get_current_task_pid()
}

pub fn get_child_task_return_val(pid: usize) -> Option<u32> {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.get_child_task_return_val(pid)
}

pub struct TaskManager {
    tasks: Vec<TaskContext>,
    current_task: usize,
    started: bool,
    time_quant: Duration,
}

impl TaskManager {
    pub fn new(time_quant: Duration) -> Self {
        Self {
            tasks: Vec::new(),
            current_task: 0,
            started: false,
            time_quant,
        }
    }

    pub fn add_task(&mut self, mut task: TaskContext) -> Result<u64, TaskError> {
        if self.tasks.len() >= MAX_TASK_COUNT {
            return Err(TaskError::TaskLimitReached);
        }
        task.state = TaskStates::Running;
        for (index, item) in self.tasks.iter_mut().enumerate() {
            if let TaskStates::Dead = item.state {
                core::mem::swap(item, &mut task);
                drop(task);
                return Ok(index as u64);
            }
        }
        let index = self.tasks.len();
        self.tasks.push(task);
        Ok(index as u64)
    }

    pub fn get_task(&mut self, pid: usize) -> Result<&mut TaskContext, TaskError> {
        let task = self
            .tasks
            .get_mut(pid)
            .ok_or(TaskError::InvalidTaskReference)?;

        Ok(task)
    }

    pub fn get_current_task(&mut self) -> &mut TaskContext {
        &mut self.tasks[self.current_task]
    }

    pub fn get_child_task_return_val(&mut self, pid: usize) -> Option<u32> {
        self.tasks[self.current_task]
            .children_return_vals
            .remove(&pid)
    }

    fn get_two_tasks(
        &mut self,
        first_task_pid: usize,
        second_task_pid: usize,
    ) -> Result<(&mut TaskContext, &mut TaskContext), TaskError> {
        if self.tasks.len() < 2
            || self.tasks.len() <= first_task_pid
            || self.tasks.len() <= second_task_pid
            || first_task_pid == second_task_pid
        {
            return Err(TaskError::ChangeTaskError);
        }

        let min_pid = if first_task_pid < second_task_pid {
            first_task_pid
        } else {
            second_task_pid
        };

        let (left, right) = self.tasks.split_at_mut(min_pid + 1);

        if first_task_pid < second_task_pid {
            Ok((
                &mut left[first_task_pid],
                &mut right[second_task_pid - first_task_pid - 1],
            ))
        } else {
            Ok((
                &mut right[first_task_pid - second_task_pid - 1],
                &mut left[second_task_pid],
            ))
        }
    }

    pub fn switch_task(&mut self) {
        if !self.started {
            return;
        }
        let previous_task_pid = self.current_task;
        let mut next_task_pid = self.current_task + 1;

        loop {
            if next_task_pid >= self.tasks.len() {
                next_task_pid = 0;
            }
            if let TaskStates::Running = self.tasks[next_task_pid].state {
                break;
            }
            next_task_pid += 1;
        }

        if self.current_task == next_task_pid {
            return;
        }

        self.current_task = next_task_pid;

        let (current_task, next_task) = self
            .get_two_tasks(previous_task_pid, next_task_pid)
            .expect("Error during task switch: {:?}");

        // #Safety: lifetime of this reference is the same as lifetime of whole TaskManager; exception_context is always properly initialized if task is in tasks vector
        unsafe {
            cpu_switch_to(
                current_task as *const _ as u64,
                next_task as *const _ as u64,
                current_task.memory_manager.additional_table_hack.as_mut() as *mut _ as u64,
                next_task.memory_manager.additional_table_hack.as_mut() as *mut _ as u64,
            );
        }
    }

    pub fn start(&mut self) {
        self.started = true;
        let task = self
            .tasks
            .get_mut(0)
            .expect("Error during scheduler start: task 0 not found");

        unsafe {
            cpu_switch_to_first(
                &task.gpr as *const _ as u64,
                task.memory_manager.additional_table_hack.as_mut() as *mut _ as u64,
            );
        }
    }

    pub fn finish_task(&mut self, return_value: u32, task_pid: usize) {
        self.tasks[task_pid].state = TaskStates::Dead;
        let keys = self.tasks[task_pid]
            .children_return_vals
            .keys()
            .cloned()
            .collect::<Vec<usize>>();
        for pid in keys {
            if pid < self.tasks.len() {
                if let TaskStates::Dead = self.tasks[pid].state {
                } else {
                    self.finish_task(special_return_vals::PARENT_PROCESS_ENDED, pid);
                }
            }
        }

        if let Some(ppid) = self.tasks[task_pid].ppid {
            self.tasks[ppid]
                .children_return_vals
                .insert(self.current_task, return_value);
        };
        self.switch_task()
    }

    pub fn finish_current_task(&mut self, return_value: u32) {
        self.finish_task(return_value, self.current_task);
    }

    pub fn get_current_task_pid(&self) -> usize {
        self.current_task
    }
}
extern "C" {
    /// Change CPU context from prev task to next task
    #[no_mangle]
    fn drop_el0();
}
// #[no_mangle]
// #[inline(never)]
// pub fn drop_el0() {
//     crate::println!("dupa");
//     unsafe {
//         llvm_asm!("brk 0");
//     };
//     loop{}
// }

pub fn handle_new_task_syscall(e: &mut ExceptionContext) {
    let function_address = e.gpr[0] as usize;
    let ptr = e.gpr[1] as *const &[u8];
    let len = e.gpr[2] as usize;

    let args: &[&[u8]] = unsafe { core::slice::from_raw_parts(ptr, len) };

    let function = unsafe {
        core::mem::transmute::<usize, extern "C" fn(usize, *const &[u8]) -> u32>(function_address)
    };
    let mut task = TaskContext::new(function, args, false).expect("Failed to create new task");

    task.ppid = Some(get_current_task_pid());

    e.gpr[0] = match add_task(task) {
        Ok(pid) => pid,
        Err(error) => {
            crate::println!("Error when creating new task: {:?}", error);
            !0u64
        }
    };
}

#[no_mangle]
pub extern "C" fn schedule_tail() {
    crate::interupts::handlers::end_scheduling();
}

#[no_mangle]
pub extern "C" fn finalize_task(returned_value: u32) {
    crate::syscall::finish_task(returned_value);
}

global_asm!(include_str!("scheduler/change_task.S"));
