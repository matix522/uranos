use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;
use core::slice;
use core::str::from_utf8;
use num_traits::FromPrimitive;

use crate::utils::ONLY_MSB_OF_USIZE;

pub fn open(filename: &str, with_write: bool) -> Result<usize, vfs::FileError> {
    let val: usize;
    let bytes = filename.as_bytes();

    unsafe {
        val = syscall3(
            bytes.as_ptr() as usize,
            bytes.len(),
            with_write as usize,
            Syscalls::OpenFile as usize,
        );
    }
    if val & ONLY_MSB_OF_USIZE > 0 {
        Err(
            vfs::FileError::from_usize(val & !ONLY_MSB_OF_USIZE).unwrap_or_else(|| {
                panic!(
                    "Unknown error during file opening: {}",
                    val & !ONLY_MSB_OF_USIZE
                )
            }),
        )
    } else {
        Ok(val)
    }
}

pub fn handle_open(context: &mut ExceptionContext) {
    let ptr = context.gpr[0] as *const u8;
    let len = context.gpr[1] as usize;
    let with_write = context.gpr[2] != 0;

    let data = unsafe { slice::from_raw_parts(ptr, len) };

    let string = from_utf8(data);

    if string.is_err() {
        crate::println!(
            "[Syscall Fault (Write)] String provided doesen't apper to be correct UTF-8 string.
            \n\t -- Caused by: '{}'",
            string.err().unwrap()
        );
        return;
    }
    let filename = string.unwrap();

    let opened_file = vfs::open(&filename, with_write);

    if opened_file.is_err() {
        context.gpr[0] = (ONLY_MSB_OF_USIZE | opened_file.err().unwrap() as usize) as u64;
        return;
    }

    let opened_file = opened_file.unwrap();

    let current_task = crate::scheduler::get_current_task_context();

    unsafe {
        context.gpr[0] = (*current_task).file_descriptor_table.add_file(opened_file) as u64;
    }
}
