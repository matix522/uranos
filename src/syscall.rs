pub use num_traits::FromPrimitive;

pub mod print;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum Syscalls {
    StartScheduling,
    Print,
    Yield,
    FinishTask,
}

#[inline(never)]
// # Safety: we are fully in control of the synchronous interrupts handling 
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
// # Safety: we are fully in control of the synchronous interrupts handling 
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
// # Safety: we are fully in control of the synchronous interrupts handling 
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
// # Safety: we are fully in control of the synchronous interrupts handling 
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
// # Safety: we are fully in control of the synchronous interrupts handling 
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
// # Safety: we are fully in control of the synchronous interrupts handling 
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

