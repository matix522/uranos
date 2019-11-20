use crate::println;
pub static mut counter2 : u64 = 0;
use crate::userspace::syscall::*;
use crate::userspace::Syscalls;

#[no_mangle]
pub extern "C" fn init() {
    loop {
        // println!("Hello from init task! ");
        // for _i in 1..1_000 {
        //     unsafe {
        //         asm! {"nop" :::: "volatile"}
        //     }
        // }
        
        unsafe {
            for _i in 1..100_000 {
                asm! {"nop" :::: "volatile"};
            }
            counter2 += 1;
        }
        let msg = "Hello from init task!";
        write(msg);
    }
}

#[no_mangle]
pub extern "C" fn test_task() {
    let mut counter: u32 = 0;
    loop {
     //   println!("Hello from test task! {}", counter);
        counter +=1;
        if counter > 10 { counter = 0; }
        for _i in 1..1_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
        unsafe {asm!{"brk #0" :::: "volatile"}}
        let msg = "Writing by syscall now\n";
        write(msg);
    }
}

#[no_mangle]
pub extern "C" fn test_task2() {
    loop {
        println!("Hello from test task number two! {}", unsafe { counter2 } );
        for _i in 1..1_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
    }
}
