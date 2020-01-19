use crate::println;

use crate::userspace::mutex::Mutex;
// use crate::sync::mutex::Mutex;
pub static mut COUNTER2: u64 = 0;

static NAMES : [&'static str; 5] = [
    "Platon",
    "Aristoteles",
    "Konfucjusz",
    "Nitze",
    "Schopenhauer"
];
use core::sync::atomic::*;
static IDs : [AtomicBool; 5] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false)
];
type Fork = Mutex<()>;
static forks : [Fork; 5] = [
    Fork::new(()),
    Fork::new(()),
    Fork::new(()),
    Fork::new(()),
    Fork::new(())
];

#[no_mangle]
pub extern "C" fn init() {
    crate::uprintln!("Creating Philisophers task");
    for _i in 0..5 {
        crate::userspace::syscall::new_task(philisopher, 0);
    }
}

#[no_mangle]
pub extern "C" fn philisopher() {

    let my_id = 'get: loop {
        for (i, atomic) in IDs.iter().enumerate() {
            if let Ok(_) = atomic.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst){
                 break 'get i;
            }
        }
    };
    let left_fork = my_id;
    let rigth_fork = (left_fork + 1) % 5; 
    crate::uprintln!("I'm {} and I have forks {} and {}", NAMES[my_id], left_fork, rigth_fork);
    let mut counter = 0;
    let mut times : u64 = 0;
    unsafe { loop {
        let start = crate::userspace::syscall::get_time();
        let (first, second) = if my_id % 2 == 0 {
            (forks[left_fork].lock(), forks[rigth_fork].lock())
        }
        else {
            (forks[rigth_fork].lock(), forks[left_fork].lock())
        };
        let end = crate::userspace::syscall::get_time();
        times += end - start;
        if counter > 10_000 {
            crate::uprintln!("[{}] Avg time {} us", NAMES[my_id], times  * 100 / (crate::userspace::syscall::get_frequency() as u64));

            counter = 0;
            times = 0;
        }
        else {
            counter += 1;
        }

        drop(first);
        drop(second);
    }}

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
