use crate::println;
pub static mut counter2 : u64 = 0;
#[no_mangle]
pub extern "C" fn init() {
    loop {
        // println!("Hello from init task! ");
        // for _i in 1..1_000 {
        //     unsafe {
        //         asm! {"nop" :::: "volatile"}
        //     }
        // }
        loop {
            
            unsafe { for _i in 1..100_000 {
                asm! {"nop" :::: "volatile"};
            }
            counter2 += 1; }
        }
    }
}

#[no_mangle]
pub extern "C" fn test_task() {
    let mut counter: u32 = 0;
    loop {
        println!("Hello from test task! {}", counter);
        counter += 1;
        for _i in 1..1_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
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
