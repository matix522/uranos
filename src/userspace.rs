mod print;

use core::sync::atomic::{AtomicU64};

mod neofetch;
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::{Ordering};
use crate::{uprint, uprintln};

#[no_mangle]
#[inline(never)]
pub extern "C" fn _true(_argc: usize, _argv: *const &[u8]) -> u32 {
    0
}
#[no_mangle]
#[inline(never)]
pub extern "C" fn _false(_argc: usize, _argv: *const &[u8]) -> u32 {
    1
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn simple_cat(argc: usize, argv: *const &[u8]) -> u32 {
    use crate::syscall::files::{ File};
    use crate::vfs::FileError;

    use core::str::from_utf8;


    let f : File = if argc == 1 {
        let args = unsafe { core::slice::from_raw_parts(argv, argc) };

        let filename = match from_utf8(args[0]) {
            Ok(val) => val,
            Err(_) => {
                uprintln!("Expected valid utf8 string");
                return 2;
            }
        };
    
        match File::open(filename, false) {
            Ok(f) => f,
            Err(e) => {
                uprintln!("A file error occured during open: {:?}", e);
                return 3;
            }
        }
    } else {
        File::get_stdin()
    };


    let mut buffer = [0u8; 64];
    loop {
        let count = match f.read(64, &mut buffer) {
            Ok(val) => val,
            Err(FileError::ReadOnClosedFile) => break,
            Err(e) => {
                uprintln!("A file error occured during read: {:?}", e);
                return 4;
            }
        };
        if count == 0 {
            break;
        }
        File::get_stdout().write(&buffer[0..count]);
    }

    f.close();
    0
}

//work in progress

#[no_mangle]
#[inline(never)]
pub extern "C" fn simple_wc(argc: usize, argv: *const &[u8]) -> u32 {
    use crate::syscall::files::File;
    use crate::syscall::*;
    use alloc::vec::Vec;
    use core::str::from_utf8;

    if argc != 1 {
        uprintln!("Invalid number of arguments");
        return 1;
    }

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    let option = match from_utf8(args[0]) {
        Ok(val) => val,
        Err(_) => {
            uprintln!("Valid options are: -c ");
            return 2;
        }
    };

    let mut buffer = [0u8; 32];
    let mut result = Vec::<u8>::new();
    loop {
        match File::get_stdin().read(32, &mut buffer) {
            Ok(res) => {
                if res > 0 {
                    result.extend_from_slice(&buffer);
                } else {
                    yield_cpu();
                }
            }
            Err(_) => break,
        };
    }

    let string = from_utf8(&result[..]).unwrap().trim_matches(char::from(0));

    let res = match option {
        "-c" => string.chars().count(),
        "-w" => {
            uprintln!("not implemented yet");
            return 10;
        }
        &_ => {
            uprintln!("not implemented yet");
            return 10;
        }
    };
    File::get_stdout().write(&format!("{}", res).as_bytes());
    0
}

#[link_section = ".task_local"]
static MY_PID: AtomicU64 = AtomicU64::new(0);
#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task(_argc: usize, _argv: *const &[u8]) -> u32 {
    use crate::syscall::*;

    let a_pid = create_task(double_chars, &[], true, None);
    let b_pid = create_task(double_chars, &[], false, Some(a_pid));
   
    loop{}
 }

pub extern "C" fn double_chars(_argc: usize, _argv: *const &[u8]) -> u32{
    use crate::syscall::files::File;
    use crate::syscall::*;
    use core::str::from_utf8;

    let mut buffer = [0u8; 2];

    let pid = get_pid();

    loop{
        let data = match File::get_stdin().read(2, &mut buffer){
            Ok(size) => size,
            Err(_) => break,
        };
        let string = match from_utf8(&buffer){
            Ok(s) => s,
            Err(_) => "ERROR",
        };

        File::get_stdout().write(string.as_bytes());
        File::get_stdout().write(string.as_bytes());
        buffer[0]=0;
        buffer[1] = 0;
      
    }
    uprintln!("NO I ELO {}\n", pid);
    0
}

pub extern "C" fn B(_argc: usize, _argv: *const &[u8]) -> u32{
    use crate::syscall::files::File;
    use core::str::from_utf8;

    let mut buffer = [0u8; 2];

    loop{
        let data = match File::get_stdin().read(2, &mut buffer){
            Ok(size) => size,
            Err(_) => break,
        };
        let string = match from_utf8(&buffer){
            Ok(s) => s,
            Err(_) => "ERROR",
        };
        File::get_stdout().write(string.as_bytes());
    }
    0
}

pub extern "C" fn test_async_files(_argc: usize, _argv: *const &[u8]) -> u32 {
    use crate::syscall::files::File;
    use crate::syscall::*;
    use crate::utils::ONLY_MSB_OF_USIZE;
    use crate::vfs;
    use core::str::from_utf8;

    let submission_buffer = get_async_submission_buffer();
    let completion_buffer = get_async_completion_buffer();

    let mut str_buffer = [0u8; 20];
    let mut str_buffer1 = [0u8; 20];

    File::async_open("file1", true, 1, submission_buffer)
        .then_read(
            20,
            &mut str_buffer as *mut [u8] as *mut u8,
            2,
            submission_buffer,
        )
        .then_seek(-15, vfs::SeekType::FromCurrent, 3, submission_buffer)
        .then_write(b"<Added>", 4, submission_buffer)
        .then_seek(2, vfs::SeekType::FromBeginning, 5, submission_buffer)
        .then_read(
            20,
            &mut str_buffer1 as *mut [u8] as *mut u8,
            6,
            submission_buffer,
        )
        .then_close(7, submission_buffer);

    asynchronous::async_print::async_print("Hello world!", 69, submission_buffer);

    loop {
        match asynchronous::async_syscall::get_syscall_returned_value(completion_buffer) {
            Some(val) => {
                uprintln!(
                    "Received response for id: {} - {} : {}",
                    val.id,
                    val.value,
                    val.value & !ONLY_MSB_OF_USIZE
                );
                if val.id == 7 {
                    let string = from_utf8(&str_buffer).unwrap();
                    uprintln!("1st Read_value: {}", string);
                    let string = from_utf8(&str_buffer1).unwrap();
                    uprintln!("2nd Read_value: {}", string);
                    loop {}
                }
            }
            None => (),
        };
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello_world(_: usize, _: *const &[u8]) -> u32 {
    uprintln!("Hello, World!");
    return 0;
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _loop(_: usize, _: *const &[u8]) -> u32 {
    loop {}
}

pub extern "C" fn pwd(_: usize, _: *const &[u8]) -> u32 {
    uprintln!("/");
    0
}
pub extern "C" fn clear(_: usize, _: *const &[u8]) -> u32 {
    uprint!("\x1B[2J\x1B[2;1H\x1B[2J\x1B[2;1H");
    0
}
pub extern "C" fn neofetch(_: usize, _: *const &[u8]) -> u32 {
    uprint!("{}", neofetch::NEOFETCH_STRING);
    0
}
pub mod shell;

type Program = (&'static str, extern "C" fn(usize, *const &[u8]) -> u32);

const PROGRAMS: [Program; 12] = [
    ("ush", ushell),
    ("loop", _loop),
    ("first_task", first_task),
    ("test_async_files", test_async_files),
    ("wc", simple_wc),
    ("cat", simple_cat),
    ("true", _true),
    ("false", _false), 
    ("pwd", pwd),
    ("clear", clear),
    ("neofetch", neofetch),
    ("hello_world", hello_world)
];

pub extern "C" fn ushell(argc: usize, argv: *const &[u8]) -> u32 {
    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    match shell::shell_impl(args) {
        Ok(_) => 0,
        Err(error_code) => error_code,
    }
}
