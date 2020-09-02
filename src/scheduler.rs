pub mod task_context;
pub mod task_stack;

use crate::device_driver;
use crate::interupts::ExceptionContext;
use alloc::vec::Vec;
use core::time::Duration;
use task_context::*;

pub const MAX_TASK_COUNT: usize = 16;

device_driver!(
    unsynchronized TASK_MANAGER: TaskManager = TaskManager::new(Duration::from_millis(100))
);

pub fn add_task(task: TaskContext) -> Result<(), TaskError> {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.add_task(task)
}

pub fn switch_task(e: &mut ExceptionContext) -> &mut ExceptionContext {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.switch_task(e)
}

pub fn start() -> &'static mut ExceptionContext {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.start()
}

pub fn get_time_quant() -> Duration {
    let scheduler = TASK_MANAGER.lock();
    scheduler.time_quant
}

pub fn finish_current_task(e: &mut ExceptionContext) -> &mut ExceptionContext {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.finish_current_task(e)
}

pub struct TaskManager {
    tasks: Vec<TaskContext>,
    current_task: usize,
    started: bool,
    time_quant: Duration,
}

impl TaskManager {
    pub const fn new(time_quant: Duration) -> Self {
        Self {
            tasks: Vec::new(),
            current_task: 0,
            started: false,
            time_quant,
        }
    }

    pub fn add_task(&mut self, mut task: TaskContext) -> Result<(), TaskError> {
        if self.tasks.len() >= MAX_TASK_COUNT {
            return Err(TaskError::TaskLimitReached);
        }
        task.state = TaskStates::Running;
        for t in &mut self.tasks {
            if let TaskStates::Dead = t.state {
                core::mem::swap(t, &mut task);
                drop(task);
                return Ok(());
            }
        }
        self.tasks.push(task);
        Ok(())
    }

    pub fn get_task(&mut self, pid: usize) -> Result<&mut TaskContext, TaskError> {
        let task = self
            .tasks
            .get_mut(pid)
            .ok_or(TaskError::InvalidTaskReference)?;

        Ok(task)
    }

    fn get_two_tasks(&mut self, first_task_pid: usize, second_task_pid: usize)-> Result<(&mut TaskContext, &mut TaskContext), TaskError>{
        if self.tasks.len() < 2 
        || self.tasks.len() <= first_task_pid 
        || self.tasks.len() <= second_task_pid 
        || first_task_pid == second_task_pid {
            return Err(TaskError::ChangeTaskError);
        }

        let min_pid = if first_task_pid < second_task_pid {first_task_pid} else {second_task_pid};

        let (left, right) = self.tasks.split_at_mut(min_pid + 1);

        if first_task_pid < second_task_pid {
            Ok((&mut left[first_task_pid], &mut right[second_task_pid - first_task_pid - 1]))
        } else {
            Ok((&mut right[first_task_pid - second_task_pid - 1], &mut left[second_task_pid]))
        }

    }

    pub fn switch_task<'a>(
        &mut self,
        current_context: &'a mut ExceptionContext,
    ) -> &'a mut ExceptionContext {
        if !self.started {
            return current_context;
        }
        let previous_task_pid = self.current_task;
        let mut next_task_pid = self.current_task+1;
        
        loop{
            if next_task_pid >= self.tasks.len() {
                next_task_pid = 0;
            }
            crate::println!("WE ARE AT {} ", next_task_pid);
            if let TaskStates::Running = self.tasks[next_task_pid].state {
                break;
            }
            next_task_pid = next_task_pid + 1;
        }

        if self.current_task == next_task_pid {
            return current_context;
        }

        self.current_task = next_task_pid;
        
        let (current_task, next_task) = self
            .get_two_tasks(previous_task_pid, next_task_pid)
            .expect("Error during task switch: {:?}");


        current_task.exception_context = current_context as *mut ExceptionContext;


        // #Safety: lifetime of this reference is the same as lifetime of whole TaskManager; exception_context is always properly initialized if task is in tasks vector
        unsafe { &mut *next_task.exception_context }
    }

    pub fn start(&mut self) -> &'static mut ExceptionContext {
        self.started = true;
        let task = self
            .tasks
            .get_mut(0)
            .expect("Error during scheduler start: task 0 not found");
        unsafe { &mut *task.exception_context }
    }

    pub fn finish_current_task<'a>(&mut self, context: &'a mut ExceptionContext) -> &'a mut ExceptionContext {
        self.tasks[self.current_task].state = TaskStates::Dead;
        self.switch_task(context)
    }
}

#[no_mangle]
#[inline(never)]
fn drop_el0() {
    unsafe {
        llvm_asm!("brk 0");
    };
}
#[no_mangle]
#[inline(never)]
pub extern "C" fn foo() {
    crate::syscall::create_task(foobar);
    loop {
        // crate::println!("BEHOLD! FIRST TASK");
        crate::syscall::print::print("BEHOLD! FIRST TASK FROM USERSPACE!!!!\n");
        // crate::syscall::yield_cpu();
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn bar() {
    loop {
        crate::println!("BEHOLD! SECOND TASK");
        crate::syscall::finish_task();
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn foobar() {
    loop {
        crate::syscall::print::print("BEHOLD! THIRD TASK");
    }
}

pub fn sample_change_task(_e: &mut ExceptionContext, is_kernel: bool) -> &mut ExceptionContext {
    let task = match TaskContext::new(if is_kernel { foo } else { bar }, is_kernel) {
        Ok(t) => t,
        Err(err) => {
            crate::println!(">>>>>> ERROR CREATING TASK CONTEXT {:?}", err);
            loop {}
        }
    };

    let boxed_task = alloc::boxed::Box::new(task);
    let task_ref: &'static TaskContext = alloc::boxed::Box::leak(boxed_task);
    // # Safety: this line can be reached only if exeption_context is allocated properly and it's memory is leaked, so it has static lifetime.
    unsafe { &mut *task_ref.exception_context }
}



pub fn handle_new_task_syscall(function_address: usize){
    crate::println!("NEW TASK FUNCTION ADDRESS {:#018x}", function_address);
    let function = unsafe {&*(function_address as *const usize as *const extern "C" fn())};

    let task = TaskContext::new(*function, false).expect("Failed to create new task");

    match add_task(task) {
        Ok(()) =>{},
        Err(error) => panic!(&format!("Error when creating new task: {:?}", error)),
    }

}