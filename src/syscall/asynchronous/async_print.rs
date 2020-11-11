use super::*;
use crate::syscall::print;
use crate::utils::circullar_buffer::*;
use async_syscall::*;

pub fn async_print_standalone(msg: &str, id: usize) {
    let submission_buffer = crate::syscall::get_async_submission_buffer();
    async_print(msg, id, submission_buffer);
}

pub fn async_print(msg: &str, id: usize, submission_buffer: &mut CircullarBuffer) {
    let bytes = msg.as_bytes();

    let a: AsyncSyscall = AsyncSyscall {
        data_size: bytes.len(),
        syscall_type: crate::syscall::asynchronous::async_syscall::AsyncSyscalls::Print,
        data: bytes,
        id,
    };

    crate::syscall::asynchronous::async_syscall::send_async_syscall(submission_buffer, a);
}

pub fn handle_async_print(ptr: *const u8, len: usize) -> usize {
    let string = unsafe { print::construct_utf8_str(ptr, len) };

    if let Some(message) = string {
        crate::print!("{}", message)
    }
    0usize
}
