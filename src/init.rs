use crate::println;
pub static mut COUNTER2: u64 = 0;
#[no_mangle]
pub extern "C" fn init() {
    crate::uprintln!("Creating second task");
    crate::userspace::syscall::new_task(test_task, 0);
    loop {
        unsafe {
            for _i in 1..1_000_000 {
                asm! {"nop" :::: "volatile"};
            }
            COUNTER2 += 1;
        }
        crate::uprintln!("HEheheheeehh {}", 8);
        // let msg = "Hello from init task!";
        // write(msg);
    }
}

#[no_mangle]
pub extern "C" fn test_task() {
    crate::uprintln!("Hello i'm new task, I will count up to 3");
    for counter in 0..4 {
        for _i in 1..1_000_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }

        crate::uprintln!("Writing by syscall now counter {}", counter);
    }
    crate::uprintln!("I'm done creating new counter");
    crate::userspace::syscall::new_task(test_task, 0);
    crate::uprintln!("Bye");
}

#[no_mangle]
pub extern "C" fn test_task2() {
    loop {
        println!("Hello from test task number two! {}", unsafe { COUNTER2 });
        for _i in 1..1_000_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
    }
}
