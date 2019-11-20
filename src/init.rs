use crate::println;
pub static mut COUNTER2: u64 = 0;
#[no_mangle]
pub extern "C" fn init() {
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
    let mut counter: u32 = 0;
    loop {
        counter += 1;
        if counter > 10 {
            counter = 0;
        }
        for _i in 1..1_000_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }

        crate::uprintln!("Writing by syscall now counter {}", counter);
    }
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
