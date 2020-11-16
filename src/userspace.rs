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

#[no_mangle]
#[inline(never)]
pub extern "C" fn simple_cat(argc: usize, argv: *const &[u8]) -> u32 {
    use crate::syscall::files::File;
    use crate::syscall::*;
    use core::convert::TryInto;
    use core::str::from_utf8;

    if argc != 1 && argc != 2 {
        print::print("Invalid number of arguments\n");
        return 1;
    }

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    let filename = match from_utf8(args[0]) {
        Ok(val) => val,
        Err(_) => {
            print::print("Expected valid utf8 string as 1st argument\n");
            return 2;
        }
    };

    let out_file = if argc == 2 {
        let pipe_flag = match from_utf8(args[1]) {
            Ok(val) => val,
            Err(_) => {
                print::print("Expected valid utf8 string as 2nd argument\n");
                return 2;
            }
        };
        match pipe_flag {
            "1" => File::get_pipeout(),
            &_ => File::get_stdout(),
        }
    } else {
        File::get_stdout()
    };

    let f = match File::open(filename, false) {
        Ok(f) => f,
        Err(e) => {
            print::print(&format!("A file error occured during open: {:?}\n", e));
            return 3;
        }
    };

    let mut buffer = [0u8; 64];
    loop {
        let count = match f.read(64, &mut buffer) {
            Ok(val) => val,
            Err(e) => {
                print::print(&format!("A file error occured during read: {:?}\n", e));
                return 4;
            }
        };
        if count == 0 {
            break;
        }
        out_file.write(&buffer[0..count]);
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
    use core::convert::TryInto;
    use core::str::from_utf8;

    if argc != 2 && argc != 3 {
        print::print("Invalid number of arguments\n");
        return 1;
    }

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    let option = match from_utf8(args[0]) {
        Ok(val) => val,
        Err(_) => {
            print::print("Valid options are: -c \n");
            return 2;
        }
    };

    let in_file = {
        let in_file_str = match from_utf8(args[1]) {
            Ok(val) => val,
            Err(_) => {
                print::print("Invalid pipe source value \n");
                return 2;
            }
        };
        let pid = match in_file_str.parse::<u64>() {
            Ok(val) => val,
            Err(_) => {
                print::print("Invalid pipe source value \n");
                return 2;
            }
        };
        set_pipe_read_on_pid(pid);
        File::get_pipein()
    };

    let out_file = if argc == 2 {
        File::get_stdout()
    } else {
        let in_file_str = match from_utf8(args[2]) {
            Ok(val) => val,
            Err(_) => {
                print::print("Invalid pipe source value \n");
                return 2;
            }
        };
        let flag = match in_file_str.parse::<usize>() {
            Ok(val) => val,
            Err(_) => {
                print::print("Invalid pipe source value \n");
                return 2;
            }
        };
        if flag > 0 {
            File::get_pipeout()
        } else {
            File::get_stdout()
        }
    };

    let mut buffer = [0u8; 32];
    let mut result = Vec::<u8>::new();
    loop {
        match in_file.read(32, &mut buffer) {
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
    in_file.close();

    let string = from_utf8(&result[..]).unwrap().trim_matches(char::from(0));

    let res = match option {
        "-c" => string.chars().count(),
        "-w" => {
            print::print("not implemented yet");
            return 10;
        }
        &_ => {
            print::print("not implemented yet");
            return 10;
        }
    };

    out_file.write(&format!("{}", res).as_bytes());
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task(_argc: usize, _argv: *const &[u8]) -> u32 {
    use crate::alloc::string::ToString;
    use crate::syscall::asynchronous::files::AsyncFileDescriptor;
    use crate::syscall::files::File;
    use crate::syscall::*;
    use core::str::from_utf8;

    let filename = "file1";
    let cat_pid = create_task(simple_cat, &[filename, "1"]);

    for _i in 1..10 {
        yield_cpu();
    }
    let cat_pid_str = cat_pid.to_string();

    let wc_pid = create_task(simple_wc, &["-c", cat_pid_str.as_str(), "1"]);

    print::print(&format!(
        "Created hello tasks with PIDs: {}, {}\n",
        cat_pid, wc_pid
    ));
    loop {
        let ret_val = get_child_return_value(wc_pid);
        if let Some(ret) = ret_val {
            print::print(&format!("Returned value from wc: {}\n", ret));
            break;
        }
        yield_cpu();
    }

    set_pipe_read_on_pid(wc_pid);

    let mut buff = [0u8; 32];
    let ret = File::get_pipein().read(32, &mut buff);
    if ret.is_err() {
        print::print(&format!(
            "An error occured during the cat {} | wc -c execution",
            filename
        ));
    };
    let string = from_utf8(&buff[..]).unwrap().trim_matches(char::from(0));
    print::print(&format!(
        "The file {} has {} characters\n",
        filename, string
    ));

    create_task(test_async_files, &[]);

    loop {}

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

    asynchronous::async_print::async_print("Hello world!\n", 69, submission_buffer);

    loop {
        match asynchronous::async_syscall::get_syscall_returned_value(completion_buffer) {
            Some(val) => {
                print::print(&format!(
                    "Received response for id: {} - {} : {}\n",
                    val.id,
                    val.value,
                    val.value & !ONLY_MSB_OF_USIZE
                ));
                if val.id == 7 {
                    let string = from_utf8(&str_buffer).unwrap();
                    print::print(&format!("1st Read_value: {}\n", string));
                    let string = from_utf8(&str_buffer1).unwrap();
                    print::print(&format!("2nd Read_value: {}\n", string));
                    loop {}
                }
            }
            None => (),
        };
    }
}
