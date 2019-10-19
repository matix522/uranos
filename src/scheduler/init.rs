use crate::print;
use crate::println;

pub fn init(){
    while true {
        println!("Hello from init task!");
        for i in 1..10000 {
            unsafe{asm!{"nop" :::: "volatile"}}
        }
    }
}


pub fn test_task(){
    while true {
        println!("Hello from test task!");
        for i in 1..10000 {
            unsafe{asm!{"nop" :::: "volatile"}}
        }
    }
}

