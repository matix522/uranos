pub mod task_context;
pub mod task_stack;

use crate::sync::nulllock::NullLock;
use alloc::vec::Vec;
use crate::interupts::ExceptionContext;


pub fn yeet(){
    unsafe{
        llvm_asm!("svc 0");
    }
}

#[no_mangle]
pub extern "C" fn foo(){
    crate::println!("BEHOLD! SECOND TASK");
}

pub fn test_schedule(e: &mut ExceptionContext){
    crate::println!("ENTETING TASK SCHEDULE");
    let task: task_context::TaskContext = match task_context::TaskContext::new(foo, 0){
        Ok(t) => t,
        Err(_) => {
            crate::println!(">>>>>>>>ERROR");
            return;
        }
    };

    crate::println!("Created task_context");


    // e.elr_el1 = task.exception_context.elr_el1;

    crate::println!("DJSKLAJDLKSJKLD");

}

pub const MAX_TASK_COUNT: usize = 16;
pub static SCHEDULER: NullLock<Scheduler> = NullLock::new(Scheduler::new());


pub struct Scheduler{
    tasks: Vec<task_context::TaskContext>,
    current_running_task: usize, 
}

impl Scheduler{
     // Creates Scheduler
     const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_running_task: 0,
        }
    }

    // pub fn schedule(&mut self) -> Result<usize, task_context::TaskError>{
    //     let next_task_pid;
    //     let tasks = &mut self.tasks;


    // }
}