pub mod task_context;
pub mod task_stack;
pub mod special_return_vals;

use crate::device_driver;
use crate::syscall::asynchronous::async_print::*;
use crate::syscall::asynchronous::async_syscall::*;
use crate::syscall::asynchronous::files::*;
use alloc::vec::Vec;
use core::time::Duration;
use task_context::*;
use crate::alloc::collections::BTreeMap;
use crate::interupts::ExceptionContext;

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
                    AsyncSyscalls::OpenFile => {
                        let ret = open::handle_async_open(ptr, length);
                        current_task
                            .async_returns_map
                            .map
                            .insert(syscall_ret.id, (syscall_ret.syscall_type, ret));
                        ret
                    }
                    AsyncSyscalls::ReadFile => {
                        read::handle_async_read(ptr, length, &mut current_task.async_returns_map)
                    }
                    AsyncSyscalls::SeekFile => {
                        seek::handle_async_seek(ptr, length, &mut current_task.async_returns_map)
                    }
                    AsyncSyscalls::WriteFile => {
                        write::handle_async_write(ptr, length, &mut current_task.async_returns_map)
                    }
                    AsyncSyscalls::CloseFile => {
                        current_task.async_returns_map.map.remove(&syscall_ret.id);
                        close::handle_async_close(ptr, length, &mut current_task.async_returns_map)
                    }
                };

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
        crate::println!("DUPPPAAAAAA");
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

    pub fn finish_task(&mut self, return_value: u32, task_pid: usize) {
        self.tasks[task_pid].state = TaskStates::Dead;
        let keys = self.tasks[task_pid].children_return_vals.keys().cloned().collect::<Vec<usize>>();
        for pid in keys{
            if pid < self.tasks.len(){
                if let TaskStates::Dead = self.tasks[pid].state{}
                else{
                    self.finish_task(special_return_vals::PARENT_PROCESS_ENDED, pid);
                }
            }
        }

        match self.tasks[task_pid].ppid {
            Some(ppid) => {
                self.tasks[ppid].children_return_vals.insert(self.current_task, Some(return_value));
            },
            None => (),
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

#[no_mangle]
#[inline(never)]
pub fn drop_el0() {
    unsafe {
        llvm_asm!("brk 0");
    };
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task() -> u32{
    let buffer = crate::syscall::get_async_submission_buffer();
    let completion_buffer = crate::syscall::get_async_completion_buffer();

    use crate::vfs;
    use core::str::from_utf8;

    let mut str_buffer = [0u8; 20];
    let mut str_buffer1 = [0u8; 20];

    let hello_pid = crate::syscall::create_task(hello);

    crate::syscall::print::print(&format!("Created hello task with PID: {}", hello_pid));
    crate::syscall::print::print(&format!("Created hello task with PID"));
    loop {}

    // crate::syscall::asynchronous::files::open::open("file1", true, 1, buffer)
    //     .then_read(20, &mut str_buffer as *mut [u8] as *mut u8, 2, buffer)
    //     .then_seek(-15, vfs::SeekType::FromCurrent, 3, buffer)
    //     .then_write(b"<Added>", 4, buffer)
    //     .then_seek(2, vfs::SeekType::FromBeginning, 5, buffer)
    //     .then_read(20, &mut str_buffer1 as *mut [u8] as *mut u8, 6, buffer)
    //     .then_close(7, buffer);

    crate::syscall::asynchronous::async_print::async_print("Hello world!", 69, buffer);

    // loop {
    //     match crate::syscall::asynchronous::async_syscall::get_syscall_returned_value(
    //         completion_buffer,
    //     ) {
    //         Some(val) => {
    //             crate::syscall::print::print(&format!(
    //                 "Received response for id: {} - {} : {}",
    //                 val.id,
    //                 val.value,
    //                 val.value & !ONLY_MSB_OF_USIZE
    //             ));
    //             if val.id == 7 {
    //                 let string = from_utf8(&str_buffer).unwrap();
    //                 crate::syscall::print::print(&format!("1st Read_value: {}", string));
    //                 let string = from_utf8(&str_buffer1).unwrap();
    //                 crate::syscall::print::print(&format!("2nd Read_value: {}", string));
    //                 loop {}
    //             }
    //         }
    //         None => crate::syscall::print::print(&format!("No responses")),
    //     };
    // }
    return 0;
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello() -> u32{
    // loop {
        crate::syscall::print::print("SECOND task USERSPACE!!!!\n");
        // crate::syscall::yield_cpu();
    // }
    return 0x2137;
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello2() {
    loop {
        crate::syscall::print::print(&format!("HELLO!2"));
        // crate::syscall::yield_cpu();
    }
}

pub fn handle_new_task_syscall(e: &mut ExceptionContext) {
    let function_address = e.gpr[0] as usize;
    
    
    let function = unsafe { core::mem::transmute::<usize, extern "C" fn() -> u32>(function_address) };
    let mut task = TaskContext::new(function, false).expect("Failed to create new task");
    
    task.ppid = Some(get_current_task_pid());
    
    let current_task: &mut TaskContext = unsafe {&mut *(get_current_task_context())};
    current_task.children_return_vals.insert(get_current_task_pid(), None);

    
    e.gpr[0] = match add_task(task) {
        Ok((pid)) => pid,
        Err(error) => {
            crate::println!("Error when creating new task: {:?}", error);
            !0u64
        },
    };

}

#[no_mangle]
pub extern "C" fn schedule_tail() {
    crate::interupts::handlers::end_scheduling();
}

#[no_mangle]
pub extern "C" fn finalize_task(returned_value: u64){
    crate::syscall::print::print(&format!("KONIEC SMIESZKOWANIA: {}", returned_value));
    crate::syscall::finish_task(returned_value);
}


global_asm!(include_str!("scheduler/change_task.S"));
