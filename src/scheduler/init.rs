use crate::print;
use crate::println;

#[no_mangle]
pub extern "C" fn init(){
    loop {
        println!("Hello from init task!");
        for i in 1..10000 {
            unsafe{asm!{"nop" :::: "volatile"}}
        }
    }
}

#[no_mangle]
pub extern "C" fn test_task(){
    loop {
        println!("Hello from test task!");
        for i in 1..10000 {
            unsafe{asm!{"nop" :::: "volatile"}}
        }
    }
}

