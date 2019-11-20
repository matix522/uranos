use crate::println;
pub static mut counter2: u64 = 0;
#[no_mangle]
pub extern "C" fn init() {
    loop {
        unsafe {
            for _i in 1..100_000 {
                asm! {"nop" :::: "volatile"};
            }
            counter2 += 1;
        }
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
        for _i in 1..1_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
        unsafe {asm!{"brk #0" :::: "volatile"}}
        let msg = "Writing by syscall now\n";
        crate::uprintln!("Writing by syscall now counter {}", counter);
    }
}

#[no_mangle]
pub extern "C" fn test_task2() {
    loop {
        println!("Hello from test task number two! {}", unsafe { counter2 });
        for i in 1..1_000_000 {
            unsafe {
                asm! {"nop" :::: "volatile"}
            }
        }
    }
}
