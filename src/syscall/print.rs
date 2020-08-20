use super::*;
use crate::interupts::ExceptionContext;

use core::slice;
use core::str::from_utf8;

pub fn print(msg: &str) {
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

pub fn handle_print_syscall(context: &mut ExceptionContext) -> &mut ExceptionContext {
    let ptr = context.gpr[0] as *const u8;
    let len = context.gpr[1] as usize;

    // println!("{:x} {}", ptr as u64, len);
    // println!("{}", *context);

    let data = unsafe { slice::from_raw_parts(ptr, len) };

    let string = from_utf8(data);

    if string.is_err() {
        crate::println!(
            "[Syscall Fault (Write)] String provided doesen't apper to be correct UTF-8 string.
            \n\t -- Caused by: '{}'",
            string.err().unwrap()
        );
        return context;
    }
    let string = string.unwrap();

    crate::print!("{}", string);

    context
}
