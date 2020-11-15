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
    use crate::syscall::*;
    use core::convert::TryInto;
    use alloc::vec::Vec;
    if argc != 1 && argc != 2 {
        crate::syscall::print::print("Invalid number of arguments\n");
        return 1;
    }
    let args = unsafe { core::slice::from_raw_parts(argv, argc) };
    let filename = core::str::from_utf8(args[0]);
    if filename.is_err() {
        crate::syscall::print::print("Expected valid utf8 string\n");
        return 2;
    }
    let filename = filename.unwrap();

    let out_fd = if argc == 2 {
        let bytes: &[u8; 8] = args[1].try_into().unwrap();
        if u64::from_le_bytes(*bytes) > 0 {
            files::PIPEOUT
        } else{
            files::STDOUT
        }
    } else { 
        files::STDOUT
    };


    let fd = files::open::open(filename, false);
    if fd.is_err() {
        crate::syscall::print::print(&format!("A file error occured: {:?}\n", fd.err().unwrap()));
        return 3;
    }
    let fd = fd.unwrap();
    let mut buffer = [0u8; 64];
    loop 
    {
        let count = files::read::read(fd, 64, &mut buffer as *mut [u8] as *mut u8).unwrap();
        if count == 0{
            break;
        }
        files::write::write(out_fd, &buffer[0..count]);
    }
    0
}

//work in progress

#[no_mangle]
#[inline(never)]
pub extern "C" fn simple_wc(argc: usize, argv: *const &[u8]) -> u32{
    use crate::syscall::*;
    use core::convert::TryInto;
    use alloc::vec::Vec;


    if argc != 2 && argc != 3 {
        crate::syscall::print::print("Invalid number of arguments\n");
        return 1;
    }

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    let option = core::str::from_utf8(args[0]);
    if  option.is_err() {
        crate::syscall::print::print("Valid options are: -c \n");
        return 2;
    }
    let option = option.unwrap();

    let fd = if argc == 0{
        files::STDIN    
    } else{
        let bytes: &[u8; 8] = args[1].try_into().unwrap();
        let pid = u64::from_le_bytes(*bytes);
        print::print(&format!("PID of the beginning of pipe: {}\n", pid));
        set_pipe_read_on_pid(pid);
        files::PIPEIN
    };

    let out_fd = if argc == 0 {
        files::STDOUT
    } else{
        let bytes: &[u8; 8] = args[2].try_into().unwrap();
        if u64::from_le_bytes(*bytes) > 0 {
            files::PIPEOUT
        } else{
            files::STDOUT
        }
    };

    let mut buffer = [0u8; 32];
    let mut result = Vec::<u8>::new();
    loop{
        let res = crate::syscall::files::read::read(fd, 32, &mut buffer as *mut [u8] as *mut u8);
        if res.is_err(){
            break;
        } else{
            let res = res.unwrap();
            if res > 0{
                result.extend_from_slice(&buffer);
            }else{
                yield_cpu();
            }
        }
    }
    let string = core::str::from_utf8(&result[..]).unwrap().trim_matches(char::from(0));

    let char_count = string.chars().count();

    files::write::write(out_fd, &format!("Char count: {}\n", char_count).as_bytes());
    
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task(_argc: usize, _argv: *const &[u8]) -> u32 {
    use core::convert::TryInto;
    use crate::syscall::*;
    
    let cat_to_pipe = 1usize.to_le_bytes();
    let cat_args = ["file1".as_bytes(), (&cat_to_pipe) as &[u8]];
    
    let cat_pid = crate::syscall::create_task(simple_cat, &cat_args);
    yield_cpu();
    yield_cpu();
    yield_cpu();
    yield_cpu();
    let pid = cat_pid.to_le_bytes();
    let to_pipe = 0usize.to_le_bytes();
    let wc_args = ["-c".as_bytes() ,(&pid) as &[u8], (&to_pipe) as &[u8]];

    let wc_pid = crate::syscall::create_task(simple_wc, &wc_args);


    files::write::write(files::PIPEOUT, b"No elo");

    print::print(&format!("Created hello tasks with PIDs: {}, {}\n", cat_pid, wc_pid));
    loop {
        let ret_val = crate::syscall::get_child_return_value(wc_pid);
        if ret_val & crate::utils::ONLY_MSB_OF_USIZE == 0 {
            print::print(&format!("Returned value from wc: {}\n", ret_val));
            break;
        }
    }

    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn pipe_end(argc: usize, argv: *const &[u8]) -> u32 {
    use core::convert::TryInto;
    use core::str::from_utf8;
    use crate::syscall::*;


    if argc != 1 {
        crate::syscall::print::print("Wrong number of args");
    }
    let args = unsafe { core::slice::from_raw_parts(argv, argc) };
    let bytes: &[u8; 8] = args[0].try_into().unwrap();
    let pid = u64::from_le_bytes(*bytes);
    print::print(&format!("PID of the beginning of pipe: {}\n", pid));
    set_pipe_read_on_pid(pid);


    let mut read_buffer = [0u8;32];
    files::read::read(files::PIPEIN, 32, &mut read_buffer as *mut [u8] as *mut u8);
    let string = from_utf8(&read_buffer).unwrap();
    print::print(string);
    // print::print(&format!("{:?}", read_buffer));
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello2() {
    loop {
        crate::syscall::print::print("HELLO!2");
        // crate::syscall::yield_cpu();
    }
}
