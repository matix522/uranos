
// #![deny(missing_docs)]
// #![deny(warnings)]

use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::print;
use crate::println;

pub mod init;

extern "C" {
    fn cpu_switch_to(prev_task_addr : u64, next_task_addr : u64) -> ();
    fn new_task_func() -> ();
}

/// Error regarding tasks
#[derive(Debug)]
pub enum TaskError {
    /// Limit of tasks has been reached when trying to add next task
    TaskLimitReached,
    /// Referenced to task out of bounds of array
    InvalidTaskReference,
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
    x: [u64; 11],
    sp: u64,
    lr: u64,
}

#[repr(C)]
#[derive(Debug)]
/// Structure containing context of a single task
pub struct TaskContext {
    /// General Purpose Registers
    gpr: GPR,
    task_state: TaskStates,
    counter : i64,
    priority: u32,
    preemption_count: u32,
    stack: Option<Box<[u8]>>,
}

const TASK_STACK_SIZE: usize = 0x800;

use crate::sync::nulllock::NullLock;

lazy_static! {
    /// Vector of tasks
    pub static ref TASKS: NullLock<Vec<TaskContext>> = NullLock::new(Vec::new());
}
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
            counter: 0,            
            priority: 0,
            preemption_count: 0,
            stack: None,
        }
    }

    /// create new task
    pub fn new(start_function: extern "C" fn(), priority: u32) -> Self {
        let mut task = Self::empty();
        let stack = Box::new([0; TASK_STACK_SIZE]);
        task.priority = priority;
        task.gpr.x[0] = start_function as *const () as u64;
        task.gpr.lr = new_task_func as *const () as u64;
        task.gpr.sp = (*stack).as_ptr() as *const () as u64;
        task.stack = Some(stack);
        task
    }

    /// Adds task to task vector and set state to running
    pub fn start_task(mut self) -> Result<(), TaskError> {
        //self.task_state = TaskStates::Running;
        let mut tasks = TASKS.lock();
        unsafe {
            if tasks.len() >= MAX_TASK_COUNT {
                return Err(TaskError::TaskLimitReached);
            }
            tasks.push(self);
        }
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn schedule_tail() -> () {

}


/// Scheduling algorithm choosing next task and switching to it
pub fn schedule() -> () {
    let mut next_task_found : bool = false;
    let mut next_task_pid : usize = 0;

    let mut tasks = TASKS.lock();

    unsafe{
        println!("Scheduling beginning, current task PID: {}", PREVIOUS_TASK_PID);

        while !next_task_found {
            //crate::println!("{:?}", *tasks);
            
            for i in 1..tasks.len() {
                //println!("Checking {} task", i);
                let curr_task : &mut TaskContext = &mut tasks[i];
                match curr_task.task_state {
                    TaskStates::Running | TaskStates::NotStarted => {
                        if curr_task.counter > 0 {
                            curr_task.counter = 0;
                            continue;
                        } else {
                            curr_task .counter = 1;
                            next_task_pid = i;
                            next_task_found = true;
                            break;
                        }
                        
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        //PREVIOUS_TASK_PID = next_task_pid;
        println!("New task: {}", next_task_pid);
        // match tasks[next_task_pid].task_state {
        //     TaskStates::NotStarted => tasks,
        //     _ => c
        // }
        change_task(next_task_pid);
            println!("F: {} {}", next_task_pid ,PREVIOUS_TASK_PID);

    }
}

// pub static mut SCHEDULING_INITIALIZED : bool = false;




// pub fn fork() -> (){
//     let curr_task =  TASKS[PREVIOUS_TASK_PID];
//     TASKS.append(curr_task.copy());

// }


static mut PREVIOUS_TASK_PID : usize = 0;


/// Function that changes current tasks and stores context of previous one in his TaskContext structure
pub unsafe fn change_task(next : usize) -> Result<(), TaskError> {
    let mut tasks = TASKS.lock();
            println!("A: {} {}", next ,PREVIOUS_TASK_PID);

    if PREVIOUS_TASK_PID == next {return Ok(());}

            println!("B: {} {}", next ,PREVIOUS_TASK_PID);


    if PREVIOUS_TASK_PID >= tasks.len() || next >= tasks.len() { return Err(TaskError::InvalidTaskReference); }
                println!("C: {} {}", next ,PREVIOUS_TASK_PID);

    let prev_task_addr = &tasks[PREVIOUS_TASK_PID] as *const TaskContext as u64;
    let next_task_addr = &tasks[next] as *const TaskContext as u64;

    PREVIOUS_TASK_PID = next;
    println!("D: {} {}", next ,PREVIOUS_TASK_PID);
    cpu_switch_to(prev_task_addr, next_task_addr);
    println!("E: {} {}", next ,PREVIOUS_TASK_PID);

    Ok(())

}

// #[link_section = ".task"]
// static TASKS: [TaskContext; MAX_TASK_COUNT] = new_task_table(); //Default::default();// = [TaskContext::new(); MAX_TASK_COUNT];

// #[link_section = ".task.stack"]
// static TASK_STACKS: [TaskStack; MAX_TASK_COUNT] = [TaskStack::new(); MAX_TASK_COUNT] ;





global_asm!(include_str!("change_task.S"));
