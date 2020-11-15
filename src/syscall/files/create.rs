use crate::interupts::ExceptionContext;
use crate::syscall::*;
use crate::vfs;
use core::slice;
use core::str::from_utf8;
use num_traits::FromPrimitive;

use crate::utils::ONLY_MSB_OF_USIZE;

pub fn create(filename: &str) -> Result<usize, vfs::FileError> {
    let val: usize;
    let bytes = filename.as_bytes();

    unsafe {
        val = syscall2(
            bytes.as_ptr() as usize,
            bytes.len(),
            Syscalls::CreateFile as usize,
        );
    }
    if val & ONLY_MSB_OF_USIZE > 0 {
        Err(
            vfs::FileError::from_usize(val & !ONLY_MSB_OF_USIZE).unwrap_or_else(|| {
                panic!(
                    "Unknown error during file creation: {}",
                    val & !ONLY_MSB_OF_USIZE
                )
            }),
        )
    } else {
        Ok(val)
    }
}

pub fn handle_create(context: &mut ExceptionContext) {
    let ptr = context.gpr[0] as *const u8;
    let len = context.gpr[1] as usize;

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

    let val = vfs::create_file(&filename);

    if val.is_err() {
        context.gpr[0] = (ONLY_MSB_OF_USIZE | val.err().unwrap() as usize) as u64;
        return;
    }

    context.gpr[0] = 0;
}
