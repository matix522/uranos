use crate::alloc::collections::BTreeMap;
use crate::alloc::string::String;

pub static mut PROGRAMS: BTreeMap<String, extern "C" fn() -> u32> =
    BTreeMap::<String, extern "C" fn() -> u32>::new();

#[no_mangle]
#[inline(never)]
pub extern "C" fn r#true() -> u32 {
    1
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn r#false() -> u32 {
    0
}
use crate::syscall::asynchronous::*;
#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task() -> u32 {
    let buffer = crate::syscall::get_async_submission_buffer();
    let completion_buffer = crate::syscall::get_async_completion_buffer();

    use crate::vfs;
    use core::str::from_utf8;
    let mut str_buffer = [0u8; 20];
    let mut str_buffer1 = [0u8; 20];

    let hello_pid = crate::syscall::create_task(hello);

    crate::syscall::print::print(&format!("Created hello task with PID: {}\n", hello_pid));
    loop {
        let ret_val = crate::syscall::get_child_return_value(hello_pid);
        if ret_val & crate::utils::ONLY_MSB_OF_USIZE == 0 {
            crate::println!("Returned value from hello_task: {}", ret_val);
            break;
        }
    }

    files::open::open("file1", true, 1, buffer)
        .then_read(20, &mut str_buffer as *mut [u8] as *mut u8, 2, buffer)
        .then_seek(-15, vfs::SeekType::FromCurrent, 3, buffer)
        .then_write(b"<Added>", 4, buffer)
        .then_seek(2, vfs::SeekType::FromBeginning, 5, buffer)
        .then_read(20, &mut str_buffer1 as *mut [u8] as *mut u8, 6, buffer)
        .then_close(7, buffer);

    async_print::async_print("Hello world!", 69, buffer);

    loop {
        if let Some(val) = async_syscall::get_syscall_returned_value(completion_buffer) {
            crate::syscall::print::print(&format!(
                "Received response for id: {} - {} : {}\n",
                val.id,
                val.value,
                val.value & !crate::utils::ONLY_MSB_OF_USIZE
            ));
            if val.id == 7 {
                let string = from_utf8(&str_buffer).unwrap();
                crate::syscall::print::print(&format!("1st Read_value: {}\n", string));
                let string = from_utf8(&str_buffer1).unwrap();
                crate::syscall::print::print(&format!("2nd Read_value: {}\n", string));
                loop {}
            }
        };
    }
    // return 0;
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello() -> u32 {
    // loop {
    crate::syscall::print::print("SECOND task USERSPACE!!!!\n");
    // crate::syscall::yield_cpu();
    // }
    3
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello2() {
    loop {
        crate::syscall::print::print("HELLO!2");
        // crate::syscall::yield_cpu();
    }
}
