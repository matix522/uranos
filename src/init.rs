use crate::println;

// use crate::userspace::mutex::Mutex;
use crate::sync::mutex::Mutex;
pub static mut COUNTER2: u64 = 0;

static NAMES: [&str; 5] = [
    "Platon",
    "Aristoteles",
    "Konfucjusz",
    "Nitze",
    "Schopenhauer",
];
use core::sync::atomic::*;
static ID: [AtomicBool; 5] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];
type Fork = Mutex<()>;
static FORKS: [Fork; 5] = [
    Fork::new(()),
    Fork::new(()),
    Fork::new(()),
    Fork::new(()),
    Fork::new(()),
];

#[no_mangle]
pub extern "C" fn init() {
    // crate::uprintln!("Creating Philisophers task");
    for _i in 0..5 {
        crate::userspace::syscall::new_task(philisopher, 0);
    }
}
#[inline(never)]
fn who_am_i_where_do_i_go() -> usize {
    'get: loop {
        for (i, atomic) in ID.iter().enumerate() {
            if atomic.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed).is_ok()
            {
                break 'get i;
            }
        }
    }
}
#[no_mangle]
pub extern "C" fn philisopher() {
    let who_am_i = who_am_i_where_do_i_go();

    let left_fork = who_am_i;
    let rigth_fork = (left_fork + 1) % 5;

    crate::uprintln!(
        "I'm {} and I would like to have forks {} and {}",
        NAMES[who_am_i],
        left_fork,
        rigth_fork
    );

    let mut counter = 0;
    let mut times: u64 = 0;
    unsafe {
        loop {
            let start = crate::userspace::syscall::get_time();
            let (first, second) = if who_am_i % 2 == 0 {
                (FORKS[left_fork].lock(), FORKS[rigth_fork].lock())
            } else {
                (FORKS[rigth_fork].lock(), FORKS[left_fork].lock())
            };
            drop(first);
            drop(second);

            let end = crate::userspace::syscall::get_time();
            times += end - start;
            if counter > 10_000 {
                crate::uprintln!(
                    "[{}] Avg time {} ns",
                    NAMES[who_am_i],
                    times * 100_000 / (crate::userspace::syscall::get_frequency() as u64)
                );

                counter = 0;
                times = 0;
            } else {
                counter += 1;
            }
        }
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
