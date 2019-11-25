use super::Syscalls;

#[inline(never)]
pub unsafe fn syscall0(mut a: usize) -> usize {
    asm!("svc   0"
          : "={x0}"(a)
          : "{x8}"(a)
          : "x0", "x8"
          : "volatile");
    return a;
}

#[inline(never)]
pub unsafe fn syscall1(mut a: usize, b: usize) -> usize {
    asm!("svc   0"
          : "={x0}"(a)
          : "{x8}"(a), "{x0}"(b)
          : "x0", "x8"
          : "volatile");

    return a;
}

#[inline(never)]
pub unsafe fn syscall2(a: usize, b: usize, c: usize) -> usize {
    let ret : usize;
    asm!("svc   0"
          : "={x0}"(ret)
          : "{x0}"(a), "{x1}"(b), "{x8}"(c)
          : "x0", "x1", "x8"
          : "volatile");

    return ret;
}

#[inline(never)]
pub unsafe fn syscall3(mut a: usize, b: usize, c: usize, d: usize) -> usize {
    asm!("svc   0"
          : "={x0}"(a)
          : "{x8}"(a), "{x0}"(b), "{x1}"(c), "{x2}"(d)
          : "x0", "x1", "x2", "x8"
          : "volatile");

    return a;
}

#[inline(never)]
pub unsafe fn syscall4(mut a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    asm!("svc   0"
          : "={x0}"(a)
          : "{x8}"(a), "{x0}"(b), "{x1}"(c), "{x2}"(d), "{x3}"(e)
          : "x0", "x1", "x2", "x3", "x8"
          : "volatile");

    return a;
}

#[inline(never)]
pub unsafe fn syscall5(mut a: usize, b: usize, c: usize, d: usize, e: usize, f: usize) -> usize {
    asm!("svc   0"
          : "={x0}"(a)
          : "{x8}"(a), "{x0}"(b), "{x1}"(c), "{x2}"(d), "{x3}"(e), "{x4}"(f)
          : "x0", "x1", "x2", "x3", "x4", "x8"
          : "volatile");

    return a;
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
pub fn new_task_syscall(start_function: extern "C" fn(), priority_difference: u32) {
    let function_ptr = start_function as *const () as usize;
    unsafe {
        syscall2(
            function_ptr,
            priority_difference as usize,
            Syscalls::NewTask as usize,
        );
    }
}
