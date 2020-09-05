pub use num_traits::FromPrimitive;

pub mod print;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum Syscalls {
    StartScheduling,
    Print,
    Yield,
    FinishTask,
    CreateTask,
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
          : "{x8}"(p1), "{x0}"(p2), "{x1}"(p3), "{x2}"(p4), "{x3}"(syscall_type)
          : "x0", "x1", "x2", "x3", "x8"
          : "volatile");

    ret
}

#[inline(never)]
/// # Safety
/// Caller needs to assusre that syscall_type is valid number for syscall with 5 arguments
/// Arguments needs to be in approperaite values for given syscall type
pub unsafe fn syscall5(p1: usize, p2: usize, p3: usize, p4: usize, p5: usize, syscall_type: usize) -> usize {
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

pub fn finish_task(){
    unsafe {
        syscall0(Syscalls::FinishTask as usize);
    }
}


pub fn create_task(function: extern "C" fn()){
    unsafe {
        // crate::println!("FUNCTION ADDRESS: {:#018x}", function as *const () as usize);
        syscall1(function as *const () as usize, Syscalls::CreateTask as usize);
    }
}

