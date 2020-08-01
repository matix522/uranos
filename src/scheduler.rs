pub mod task_context;
pub mod task_stack;

use crate::interupts::ExceptionContext;
use crate::sync::nulllock::NullLock;
use alloc::vec::Vec;

pub const MAX_TASK_COUNT: usize = 16;
pub(super) static mut TASK_MANAGER: TaskManager = TaskManager::new();

pub fn add_task(mut task: task_context::TaskContext) -> Result<(), task_context::TaskError> {
    unsafe { TASK_MANAGER.add_task(task) }
}

pub fn switch_task(e: &mut ExceptionContext) -> &mut ExceptionContext {
    unsafe { TASK_MANAGER.switch_task(e) }
}

pub fn start() -> &'static mut ExceptionContext {
    unsafe { TASK_MANAGER.start() }
}

pub struct TaskManager {
    tasks: Vec<task_context::TaskContext>,
    current_task: usize,
}

impl TaskManager {
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_task: 0,
        }
    }

    pub fn add_task(
        &mut self,
        mut task: task_context::TaskContext,
    ) -> Result<(), task_context::TaskError> {
        if self.tasks.len() >= MAX_TASK_COUNT {
            return Err(task_context::TaskError::TaskLimitReached);
        }
        for t in &mut self.tasks {
            if let task_context::TaskStates::Dead = t.state {
                core::mem::swap(t, &mut task);
                drop(task);
                return Ok(());
            }
        }
        self.tasks.push(task);
        Ok(())
    }

    pub fn get_task(
        &mut self,
        pid: usize,
    ) -> Result<&mut task_context::TaskContext, task_context::TaskError> {
        let task = self
            .tasks
            .get_mut(pid)
            .ok_or(task_context::TaskError::InvalidTaskReference)?;

        Ok(task)
    }

    fn get_two_tasks(
        &mut self,
        split_point: usize,
    ) -> Result<
        (
            &mut task_context::TaskContext,
            &mut task_context::TaskContext,
        ),
        task_context::TaskError,
    > {
        if self.tasks.len() < 2 {
            return Err(task_context::TaskError::ChangeTaskError);
        }

        if split_point + 1 >= self.tasks.len() {
            let (left, right) = self.tasks.split_at_mut(1);
            return Ok((&mut right[split_point - 1], &mut left[0]));
        }
        let (left, right) = self.tasks.split_at_mut(split_point + 1);
        Ok((&mut left[split_point], &mut right[0]))
    }

    pub fn switch_task(&mut self, current_context: &mut ExceptionContext) -> &mut ExceptionContext {
        let split_point = self.current_task;
        self.current_task = if self.current_task + 1 >= self.tasks.len() {
            0
        } else {
            self.current_task + 1
        };
        match self.get_two_tasks(split_point) {
            Ok((current_task, next_task)) => {
                unsafe {
                    current_task.exception_context = current_context as *mut ExceptionContext;
                };

                // #Safety: lifetime of this reference is the same as lifetime of whole TaskManager; exception_context is always properly initialized if task is in tasks vector
                unsafe { &mut *next_task.exception_context }
            }
            Err(err) => {
                crate::println!("Error during task switch: {:?}", err);
                loop {}
            }
        }
    }

    pub fn start(&mut self) -> &mut ExceptionContext {
        match self.tasks.get_mut(0) {
            Some(task) => unsafe { return &mut *task.exception_context },
            None => {
                crate::println!("Error during scheduler start: task 0 not found");
                loop {}
            }
        }
    }
}

pub fn start_scheduling() {
    unsafe {
        llvm_asm!("svc 0" :: "{x8}"(1) : "x8" : "volatile");
    }
}

pub fn give_core() {
    unsafe {
        llvm_asm!("svc 0" :: "{x8}"(0) : "x8": "volatile");
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
    loop {
        crate::println!("BEHOLD! FIRST TASK");
        give_core();
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn bar() {
    loop {
        crate::println!("BEHOLD! SECOND TASK");
        give_core();
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn foobar() {
    loop {
        crate::println!("BEHOLD! THIRD TASK");
        give_core();
    }
}

pub fn sample_change_task(_e: &mut ExceptionContext, is_kernel: bool) -> &mut ExceptionContext {
    let task = match task_context::TaskContext::new(if is_kernel { foo } else { bar }, is_kernel) {
        Ok(t) => t,
        Err(err) => {
            crate::println!(">>>>>> ERROR CREATING TASK CONTEXT {:?}", err);
            loop {}
        }
    };

    let boxed_task = alloc::boxed::Box::new(task);
    let task_ref: &'static task_context::TaskContext = alloc::boxed::Box::leak(boxed_task);
    // # Safety: this line can be reached only if exeption_context is allocated properly and it's memory is leaked, so it has static lifetime.
    unsafe { &mut *task_ref.exception_context }
}
