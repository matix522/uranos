use super::Syscalls;


#[inline(never)]
pub unsafe fn syscall0(a: usize) -> usize {
    let ret: usize;
    asm!("svc   0"
          : "={x0}"(ret)
          : "{x8}"(a)
          : "x8"
          : "volatile");

    ret
}

#[inline(never)]
pub unsafe fn syscall1(a: usize, b: usize) -> usize {
    let ret: usize;
    asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(a), "{x8}"(b)
          : "x0", "x8"
          : "volatile");

    ret
}

#[inline(never)]
pub unsafe fn syscall2(a: usize, b: usize, c: usize) -> usize {
    let ret: usize;
    asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(a), "{x1}"(b), "{x8}"(c)
          : "x0", "x1", "x8"
          : "volatile");

    ret
}

#[inline(never)]
pub unsafe fn syscall3(a: usize, b: usize, c: usize, d: usize) -> usize {
    let ret: usize;
    asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(a), "{x1}"(b), "{x2}"(c), "{x8}"(d)
          : "x0", "x1", "x2", "x8"
          : "volatile");

    ret
}


#[inline(never)]
#[allow(clippy::many_single_char_names)]
pub unsafe fn syscall4(a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    let ret: usize;
    asm!("svc   0"
          : "={x0}"(ret)
          : "{x8}"(a), "{x0}"(b), "{x1}"(c), "{x2}"(d), "{x3}"(e)
          : "x0", "x1", "x2", "x3", "x8"
          : "volatile");

    ret
}

#[inline(never)]
#[allow(clippy::many_single_char_names)]
pub unsafe fn syscall5(a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    let ret: usize;
    asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(a), "{x1}"(b), "{x2}"(c), "{x3}"(d), "{x4}"(e), "{x8}"(f)
          : "x0", "x1", "x2", "x3", "x4", "x8"
          : "volatile");

    ret
}

pub fn write(msg: &str) {
    let bytes = msg.as_bytes();
    // crate::println!("{}",msg);

    unsafe {
        syscall2(
            bytes.as_ptr() as usize,
            bytes.len(),
            Syscalls::Print as usize,
        );
    }
}
/*  TODO
pub fn writeln(msg: &str){
    write(format!("{}\n", msg));
}*/
pub fn new_task(start_function: extern "C" fn(), priority_difference: usize) {
    let function_ptr = start_function as usize;

    unsafe {
        syscall2(
            function_ptr,
            priority_difference as usize,
            Syscalls::NewTask as usize,
        );
    }
}
pub fn terminate_user_task(return_value: usize) -> ! {
    unsafe {
        syscall1(return_value, Syscalls::TerminateTask as usize);
    }
    loop {}
}
pub fn get_frequency() -> u64 {
    unsafe {
        syscall0(Syscalls::GetFrequency as usize) as u64
    }
}

pub fn get_time() -> u64 {
    unsafe {
        syscall0(Syscalls::GetTime as usize) as u64
    }
}
pub fn yield_cpu() -> u64 {
    unsafe {
        syscall0(Syscalls::Yield as usize) as u64
    }
}
