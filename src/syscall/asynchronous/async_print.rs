use super::*;
use crate::syscall::print;
use crate::utils::circullar_buffer::*;
use async_syscall::*;

pub fn async_print_standalone(msg: &str) {
    let write_buffer = crate::syscall::get_async_write_buffer();
    async_print(msg, write_buffer);
}

pub fn async_print(msg: &str, write_buffer: &mut CircullarBuffer) {
    let bytes = msg.as_bytes();

    let a: AsyncSyscall = AsyncSyscall {
        data_size: bytes.len(),
        syscall_type: crate::syscall::asynchronous::async_syscall::AsyncSyscalls::Print,
        data: bytes,
    };

    crate::syscall::asynchronous::async_syscall::send_async_syscall(write_buffer, a);
}

pub fn handle_async_print(ptr: *const u8, len: usize) {
    let string = unsafe { print::construct_utf8_str(ptr, len) };

    if let Some(message) = string {
        crate::print!("{}", message)
    }
}
