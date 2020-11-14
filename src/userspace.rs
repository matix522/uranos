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
pub extern "C" fn poor_cat(argc: usize, argv: *const &[u8]) -> u32 {
    use core::str::from_utf8;
    if argc != 1 {
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
    let fd = crate::syscall::files::open::open(filename, false).unwrap();
    let mut buffer = [0u8; 32];
    while crate::syscall::files::read::read(fd, 32, &mut buffer as *mut [u8] as *mut u8).unwrap()
        > 0
    {
        let string = from_utf8(&buffer).unwrap();
        crate::syscall::print::print(string);
    }
    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn first_task(_argc: usize, _argv: *const &[u8]) -> u32 {
    let args = ["file1".as_bytes()];

    let hello_pid = crate::syscall::create_task(poor_cat, &args);

    crate::syscall::print::print(&format!("Created hello task with PID: {}\n", hello_pid));
    loop {
        let ret_val = crate::syscall::get_child_return_value(hello_pid);
        if ret_val & crate::utils::ONLY_MSB_OF_USIZE == 0 {
            crate::syscall::print::print(&format!("Returned value from hello_task: {}", ret_val));
            break;
        }
    }

    0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn hello(argc: usize, argv: *const &[u8]) -> u32 {
    crate::syscall::print::print("SECOND task USERSPACE!!!!\n");

    let args = unsafe { core::slice::from_raw_parts(argv, argc) };

    for (_index, arg) in args.iter().enumerate() {
        // let msg = core::str::from_utf8(arg).expect("invalid utf8 string");
        crate::syscall::print::print(&format!("Received argument: {:?}\n", arg));
    }
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
