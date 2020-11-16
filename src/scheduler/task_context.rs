use super::task_memory_manager;
use super::task_stack;
use crate::alloc::collections::BTreeMap;
use crate::syscall::asynchronous::async_returned_values::AsyncReturnedValues;
use crate::syscall::files::file_descriptor_map::*;
use crate::utils::circullar_buffer::*;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
/// Stack size of task in bytes
pub const TASK_STACK_SIZE: usize = 0x8000;
extern "C" {
    /// Signal end of scheduling, zero x0 - x18 and jump to x19
    fn new_task_func();

}

/// Error regarding tasks
#[derive(Debug)]
pub enum TaskError {
    /// Limit of tasks has been reached when trying to add next task
    TaskLimitReached,
    /// Stack could not be allocated
    StackAllocationFail,
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
    /// Task was ended but has data to be retrieved from pipe
    Zombie = 3,
    /// Task is dead and waiting to clean after it
    Dead = 4,
}

#[repr(C)]
pub struct Gpr {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub sp: u64,
    pub lr: u64,
    pub sp_el0: u64,
}

impl Default for Gpr {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

#[repr(C)]
pub struct TaskContext {
    pub(super) gpr: Gpr,
    pub(super) state: TaskStates,
    el0_stack: Option<task_stack::TaskStack>,
    el1_stack: Option<task_stack::TaskStack>,
    is_kernel: bool,
    pub submission_buffer: CircullarBuffer,
    pub completion_buffer: CircullarBuffer,
    pub file_descriptor_table: FileDescriptiorMap,
    pub async_returns_map: AsyncReturnedValues,
    pub children_return_vals: BTreeMap<usize, u32>,
    pub was_returned_value_read: bool,
    pub pipe_from: Option<usize>,
    pub mapped_fds: BTreeMap<usize, usize>,
    pipe_queue: VecDeque<Vec<u8>>,
    pub memory_manager: task_memory_manager::TaskMemoryManager,
    pub ppid: Option<usize>,
}

// ONLY TEMPORARY SOLUTION
unsafe impl Sync for TaskContext {}
unsafe impl Send for TaskContext {}

static NEXT_STATCK_PTR: AtomicUsize = AtomicUsize::new(0x2_0000_0000);

impl TaskContext {
    fn empty() -> Self {
        TaskContext {
            gpr: Default::default(),
            state: TaskStates::NotStarted,
            el1_stack: None,
            el0_stack: None,
            is_kernel: false,
            submission_buffer: CircullarBuffer::new(),
            completion_buffer: CircullarBuffer::new(),
            file_descriptor_table: FileDescriptiorMap::new(),
            async_returns_map: AsyncReturnedValues::new(),
            children_return_vals: BTreeMap::<usize, u32>::new(),
            was_returned_value_read: false,
            mapped_fds: BTreeMap::<usize, usize>::new(),
            pipe_queue: VecDeque::<Vec<u8>>::new(),
            memory_manager: Default::default(),
            ppid: None,
            pipe_from: None,
        }
    }

    pub fn update_zombie(&mut self) {
        if let TaskStates::Zombie = self.state {
            if self.pipe_queue.is_empty()
                && self.submission_buffer.is_empty()
                && self.was_returned_value_read
            {
                self.state = TaskStates::Dead;
            }
        }
    }

    pub fn get_item_from_pipe_queue(&mut self) -> Option<Vec<u8>> {
        let val = self.pipe_queue.pop_front();
        self.update_zombie();
        val
    }

    pub fn push_back_item_to_pipe_queue(&mut self, element: Vec<u8>) {
        self.pipe_queue.push_back(element)
    }

    pub fn push_front_item_to_pipe_queue(&mut self, element: Vec<u8>) {
        self.pipe_queue.push_front(element)
    }

    pub fn is_pipe_queue_empty(&self) -> bool {
        self.pipe_queue.is_empty()
    }
    pub fn get_state(&self) -> &TaskStates {
        &self.state
    }

    pub fn new(
        start_function: extern "C" fn(usize, *const &[u8]) -> u32,
        args: &[&[u8]],
        is_kernel: bool,
    ) -> Result<Self, TaskError> {
        let mut task: TaskContext = Self::empty();

        let user_address =
            |address: usize| ((address & !crate::KERNEL_OFFSET) | 0x1_0000_0000) as u64;

        task.is_kernel = is_kernel;

        let mut el0_stack = task_stack::TaskStack::new(
            TASK_STACK_SIZE,
            Some(NEXT_STATCK_PTR.fetch_add(TASK_STACK_SIZE * 16, Ordering::SeqCst)),
            false,
        )
        .ok_or(TaskError::StackAllocationFail)?;

        let mut el1_stack = task_stack::TaskStack::new(
            TASK_STACK_SIZE,
            Some(NEXT_STATCK_PTR.fetch_add(TASK_STACK_SIZE * 16, Ordering::SeqCst)),
            true,
        )
        .ok_or(TaskError::StackAllocationFail)?;

        let target_stack = if task.is_kernel {
            &mut el1_stack
        } else {
            &mut el0_stack
        };

        let mut argv = Vec::<&[u8]>::new();
        let mut remaining_size = target_stack.size;

        let mut target_stack_pointer = target_stack.base() as *mut u8;
        //copy the args onto task stack
        for arg in args.iter() {
            let arg_len = arg.len();
            if remaining_size <= arg_len {
                panic!("Given args does not fit in task stack");
            }
            target_stack_pointer = unsafe { target_stack_pointer.sub(arg_len) };

            unsafe {
                core::ptr::copy_nonoverlapping(
                    arg.as_ptr() as *const u8,
                    target_stack_pointer,
                    arg_len,
                );
            }
            remaining_size -= arg_len;
            let slice = unsafe { core::slice::from_raw_parts(target_stack_pointer, arg_len) };
            argv.push(slice);
        }

        //copy the args vector onto a stack
        if remaining_size <= argv.len() {
            panic!("Given args does not fit in task stack");
        }
        target_stack_pointer = unsafe { target_stack_pointer.sub(argv.len()) };

        unsafe {
            core::ptr::copy_nonoverlapping(
                argv[..].as_ptr() as *const u8,
                target_stack_pointer,
                argv.len(),
            );
        }

        task.gpr.lr = new_task_func as *const () as u64;
        task.gpr.sp = if task.is_kernel {
            target_stack_pointer as u64
        } else {
            el1_stack.base() as u64
        };

        if task.is_kernel {
            task.gpr.x19 = start_function as *const () as u64;
        } else {
            task.gpr.x19 = crate::scheduler::drop_el0 as *const () as u64;
            task.gpr.x22 = user_address(start_function as *const () as usize);
            task.gpr.sp_el0 = if task.is_kernel {
                el0_stack.base() as u64
            } else {
                target_stack_pointer as u64
            };
        }
        task.gpr.x20 = argv.len() as u64;
        task.gpr.x21 = argv[..].as_ptr() as u64;

        task.el0_stack = Some(el0_stack);
        task.el1_stack = Some(el1_stack);

        Ok(task)
    }
}
