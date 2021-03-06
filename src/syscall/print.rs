use super::*;
use crate::interupts::ExceptionContext;

use core::slice;
use core::str::from_utf8;

pub fn print(msg: &str) {
    let bytes = msg.as_bytes();

    unsafe {
        syscall2(
            bytes.as_ptr() as usize,
            bytes.len(),
            Syscalls::Print as usize,
        );
    }
}

pub fn handle_print_syscall(context: &ExceptionContext) {
    let ptr = context.gpr[0] as *const u8;
    let len = context.gpr[1] as usize;

    let string = unsafe { construct_utf8_str(ptr, len) };

    if let Some(message) = string {
        crate::print!("{}", message)
    }
}

/// # Safety user must ensure that the pointer is describing the correct region
pub unsafe fn construct_utf8_str(ptr: *const u8, len: usize) -> Option<&'static str> {
    let data = slice::from_raw_parts(ptr, len);

    let string = from_utf8(data);

    if string.is_err() {
        crate::println!(
            "[Syscall Fault (Write)] String provided doesen't apper to be correct UTF-8 string.
            \n\t -- Caused by: '{}'",
            string.err().unwrap()
        );
        return None;
    }
    string.ok()
}
