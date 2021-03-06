pub use num_traits::FromPrimitive;

pub mod files;
pub mod print;

pub mod asynchronous;

use crate::utils::circullar_buffer::*;
use crate::utils::ONLY_MSB_OF_USIZE;
use core::convert::TryFrom;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum Syscalls {
    StartScheduling,
    Print,
    Yield,
    FinishTask,
    CreateTask,
    CheckEL,
    GetAsyncSubmissionBuffer,
    GetAsyncCompletionBuffer,
    OpenFile,
    ReadFile,
    CloseFile,
    SeekFile,
    WriteFile,
    GetPID,
    GetChildReturnValue,
    CreateFile,
    DeleteFile,
    SetPipeReadOnPID,
}

#[inline(never)]
/// # Safety
/// Caller needs to assusre that syscall_type is valid number for syscall with no arguments
pub unsafe fn syscall0(syscall_type: usize) -> usize {
    let ret: usize;
    llvm_asm!("svc   0"
          : "={x0}"(ret)
          : "{x8}"(syscall_type)
          : "x8"
          : "volatile");
    ret
}

#[inline(never)]
/// # Safety
/// Caller needs to assusre that syscall_type is valid number for syscall with 1 argument
/// Argument needs to be in approperaite value for given syscall type
pub unsafe fn syscall1(p1: usize, syscall_type: usize) -> usize {
    let ret: usize;
    llvm_asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(p1), "{x8}"(syscall_type)
          : "x0", "x8"
          : "volatile");
    ret
}

#[inline(never)]
/// # Safety
/// Caller needs to assusre that syscall_type is valid number for syscall with 2 arguments
/// Arguments needs to be in approperaite values for given syscall type
pub unsafe fn syscall2(p1: usize, p2: usize, syscall_type: usize) -> usize {
    let ret: usize;
    llvm_asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(p1), "{x1}"(p2), "{x8}"(syscall_type)
          : "x0", "x1", "x8"
          : "volatile");
    ret
}

#[inline(never)]
/// # Safety
/// Caller needs to assusre that syscall_type is valid number for syscall with 3 arguments
/// Arguments needs to be in approperaite values for given syscall type
pub unsafe fn syscall3(p1: usize, p2: usize, p3: usize, syscall_type: usize) -> usize {
    let ret: usize;
    llvm_asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(p1), "{x1}"(p2), "{x2}"(p3), "{x8}"(syscall_type)
          : "x0", "x1", "x2", "x8"
          : "volatile");

    ret
}

#[inline(never)]
/// # Safety
/// Caller needs to assusre that syscall_type is valid number for syscall with 4 arguments
/// Arguments needs to be in approperaite values for given syscall type
pub unsafe fn syscall4(p1: usize, p2: usize, p3: usize, p4: usize, syscall_type: usize) -> usize {
    let ret: usize;
    llvm_asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(p1), "{x1}"(p2), "{x2}"(p3), "{x3}"(p4), "{x8}"(syscall_type)
          : "x0", "x1", "x2", "x3", "x8"
          : "volatile");

    ret
}

#[inline(never)]
/// # Safety
/// Caller needs to assusre that syscall_type is valid number for syscall with 5 arguments
/// Arguments needs to be in approperaite values for given syscall type
pub unsafe fn syscall5(
    p1: usize,
    p2: usize,
    p3: usize,
    p4: usize,
    p5: usize,
    syscall_type: usize,
) -> usize {
    let ret: usize;
    llvm_asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(p1), "{x1}"(p2), "{x2}"(p3), "{x3}"(p4), "{x4}"(p5), "{x8}"(syscall_type)
          : "x0", "x1", "x2", "x3", "x4", "x8"
          : "volatile");

    ret
}

pub fn start_scheduling() {
    unsafe {
        syscall0(Syscalls::StartScheduling as usize);
    }
}

pub fn yield_cpu() {
    unsafe {
        syscall0(Syscalls::Yield as usize);
    }
}

pub fn finish_task(return_val: u32) {
    unsafe {
        syscall1(return_val as usize, Syscalls::FinishTask as usize);
    }
}

pub fn create_task(
    function: extern "C" fn(usize, *const &[u8]) -> u32,
    args: &[&str],
    stdout_to_pipe: bool,
    pid_pipe_to_stdin: Option<u64>,
) -> u64 {
    let stdin_val = match pid_pipe_to_stdin {
        Some(val) => {
            if val == !0u64 {
                return ONLY_MSB_OF_USIZE as u64;
            }
            val as usize
        }
        None => !0usize,
    };
    unsafe {
        syscall5(
            function as *const () as usize,
            args as *const [&str] as *const () as usize,
            args.len(),
            stdout_to_pipe as usize,
            stdin_val,
            Syscalls::CreateTask as usize,
        ) as u64
    }
}
pub fn get_async_submission_buffer() -> &'static mut CircullarBuffer {
    unsafe { &mut *(syscall0(Syscalls::GetAsyncSubmissionBuffer as usize) as *mut CircullarBuffer) }
}
pub fn get_async_completion_buffer() -> &'static mut CircullarBuffer {
    unsafe { &mut *(syscall0(Syscalls::GetAsyncCompletionBuffer as usize) as *mut CircullarBuffer) }
}

pub fn get_pid() -> usize {
    unsafe { syscall0(Syscalls::GetPID as usize) as usize }
}

pub fn get_child_return_value(pid: u64) -> Option<u32> {
    let val = unsafe { syscall1(pid as usize, Syscalls::GetChildReturnValue as usize) as usize };
    if val & ONLY_MSB_OF_USIZE > 0 {
        None
    } else {
        u32::try_from(val).ok()
    }
}

pub fn set_pipe_read_on_pid(pid: u64) {
    unsafe {
        syscall1(pid as usize, Syscalls::SetPipeReadOnPID as usize);
    }
}
