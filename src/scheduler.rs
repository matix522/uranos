pub mod task_context;
pub mod task_stack;

use crate::device_driver;
use crate::syscall::asynchronous::async_print::*;
use crate::syscall::asynchronous::async_returned_values::*;
use crate::syscall::asynchronous::async_syscall::*;
use crate::syscall::asynchronous::files::*;
use alloc::vec::Vec;
use core::time::Duration;
use task_context::*;

pub const MAX_TASK_COUNT: usize = 2048;

extern "C" {
    /// Change CPU context from prev task to next task
    fn cpu_switch_to(prev_task_addr: u64, next_task_addr: u64);
    /// Change CPU context to init task (dummy lands in unused x0 for sake of simplicity)
    fn cpu_switch_to_first(init_task_addr: u64) -> !;

}

device_driver!(
    unsynchronized TASK_MANAGER: TaskManager = TaskManager::new(Duration::from_millis(100))
);

pub fn add_task(task: TaskContext) -> Result<(), TaskError> {
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

pub fn finish_current_task() {
    let mut scheduler = TASK_MANAGER.lock();
    scheduler.finish_current_task();
}

pub fn get_current_task_pid() -> usize {
    let scheduler = TASK_MANAGER.lock();
    scheduler.get_current_task_pid()
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

    pub fn get_current_task(&mut self) -> &mut TaskContext {
        &mut self.tasks[self.current_task]
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

        let mut values_map = AsyncReturnedValues::new();

        while !current_task.submission_buffer.is_empty() {
            let syscall_ret_opt = crate::syscall::asynchronous::async_syscall::read_async_syscall(
                &mut current_task.submission_buffer,
            );
            if let Some(syscall_ret) = syscall_ret_opt {
                //ommit the syscall type value that is at the beginning of the data from buffer
                let data = syscall_ret.get_syscall_data();
                let ptr = data as *const _ as *const u8;
                let length = syscall_ret.get_data_size();
                crate::println!("Handling syscall of id: {}", syscall_ret.id);
                let returned_value = match syscall_ret.syscall_type {
                    AsyncSyscalls::Print => handle_async_print(ptr, length),
                    AsyncSyscalls::OpenFile => open::handle_async_open(ptr, length),
                    AsyncSyscalls::ReadFile => {
                        read::handle_async_read(ptr, length, &mut values_map)
                    }
                };
                values_map
                    .map
                    .insert(syscall_ret.id, (syscall_ret.syscall_type, returned_value));
                let buffer_frame = current_task
                    .completion_buffer
                    .reserve(core::mem::size_of::<AsyncSyscallReturnedValue>())
                    .expect("Error during sending async syscall response");
                let return_structure: &mut AsyncSyscallReturnedValue = unsafe {
                    crate::utils::struct_to_slice::u8_slice_to_any_mut(buffer_frame.memory)
                };
                return_structure.id = syscall_ret.id;
                return_structure.value = returned_value;
            }
        }

        // #Safety: lifetime of this reference is the same as lifetime of whole TaskManager; exception_context is always properly initialized if task is in tasks vector
        unsafe {
            cpu_switch_to(
                current_task as *const _ as u64,
                next_task as *const _ as u64,
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
            cpu_switch_to_first(task as *const _ as u64);
        }
    }

    pub fn finish_current_task(&mut self) {
        self.tasks[self.current_task].state = TaskStates::Dead;
        self.switch_task()
    }

    pub fn get_current_task_pid(&self) -> usize {
        self.current_task
    }
}

#[no_mangle]
#[inline(never)]
pub fn drop_el0() {
    unsafe {
        llvm_asm!("brk 0");
    };
}
#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task() {
    let buffer = crate::syscall::get_async_submission_buffer();
    let completion_buffer = crate::syscall::get_async_completion_buffer();

    use crate::alloc::string::String;
    use crate::alloc::string::ToString;
    use crate::syscall::asynchronous::files::open::*;
    use crate::vfs;
    use core::str::from_utf8;

    // let mut future: FutureAsyncSyscallResult::<Result<usize, vfs::FileError>> = FutureAsyncSyscallResult::new();

    let mut str_buffer = [0u8; 20];

    crate::syscall::asynchronous::async_print::async_print("Hello world!", 69, buffer);

    crate::syscall::asynchronous::files::open::open("file1".to_string(), true, 1, buffer);
    crate::syscall::asynchronous::files::read::read(
        AsyncFileDescriptor::AsyncSyscallReturnValue(1),
        20,
        &mut str_buffer as *mut [u8] as *mut u8,
        2,
        buffer,
    );

    // crate::println!("{:?}, {:#018x}", future, &syscall_data.future  as *const _ as u64 );
    // crate::println!("{:?}", future);
    // crate::println!("{:?}", future);
    // crate::syscall::yield_cpu();
    // crate::println!("{:?}", future);
    // crate::println!("{:?}", future);
    // crate::println!("{:?}", future);

    loop {
        match crate::syscall::asynchronous::async_syscall::get_syscall_returned_value(
            completion_buffer,
        ) {
            Some(val) => {
                crate::println!("Received response for id: {} - {}", val.id, val.value);
                if val.id == 2 {
                    let string = from_utf8(&str_buffer).unwrap();
                    crate::println!("Read_value: {}", string);
                    loop {}
                }
            }
            None => crate::println!("No responses"),
        };
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello() {
    loop {
        // crate::syscall::print::print("SECOND task USERSPACE!!!!\n");
        crate::syscall::yield_cpu();
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello2() {
    loop {
        // crate::println!("HELLO!2");
        crate::syscall::yield_cpu();
    }
}

pub fn handle_new_task_syscall(function_address: usize) {
    // crate::println!("NEW TASK FUNCTION ADDRESS {:#018x}", function_address);
    let function = unsafe { core::mem::transmute::<usize, extern "C" fn()>(function_address) };
    let task = TaskContext::new(function, false).expect("Failed to create new task");

    match add_task(task) {
        Ok(()) => {}
        Err(error) => crate::println!("Error when creating new task: {:?}", error),
    }
}

#[no_mangle]
pub extern "C" fn schedule_tail() {
    crate::interupts::handlers::end_scheduling();
}

global_asm!(include_str!("scheduler/change_task.S"));
