use crate::print;
use crate::println;

#[no_mangle]
pub extern "C" fn init() {
    loop {
        println!("Hello from init task! ");
        for i in 1..1000000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn test_task() {
    let mut counter: u32 = 0;
    loop {
        println!("Hello from test task! {}", counter);
        counter += 1;
        for i in 1..1000000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn test_task2() {
    loop {
        println!("Hello from test task number two!");
        for i in 1..1000000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
    }
}
