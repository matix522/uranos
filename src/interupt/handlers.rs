use super::*;
use crate::interupt::ExceptionContext;
use crate::userspace::Syscalls;
pub use num_traits::FromPrimitive;
use crate::println;
use crate::print;
use crate::scheduler;
use timer::ArmQemuTimer as Timer;
use core::sync::atomic::AtomicBool;
use core::slice;
use core::str::*;

#[no_mangle]
pub unsafe extern "C" fn default_interupt_handler(context: &mut ExceptionContext, id : usize) {
    println!("Interupt Happened of ID-{}:  {:?}", id, *context);
    gpio::blink();
}


static mut is_scheduling :AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn end_scheduling() {
    unsafe{
        is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
    }
}

#[no_mangle]
pub unsafe extern "C" fn current_elx_irq(_context: &mut ExceptionContext) {
    // SECONDS += 1;
    // let sec;
    // if SECONDS == 1 {
    //     sec = "second";
    // } else {
    //     sec = "seconds";
    // }
    // println!("\x1B[s\x1B[1;1H\x1B[38;5;204mTimer interupt happened \x1B[38;5;39m{} {}\x1B[38;5;204m after startup\x1B[0m\x1B[K", SECONDS, sec);
    // print!("\x1B[u");
    //println!("Timer interupt happened {} {} after startup", SECONDS, sec);

    Timer::interupt_after(Timer::get_frequency());
    Timer::enable();
    super::enable_irqs();
    if(is_scheduling.load(core::sync::atomic::Ordering::Relaxed)) {return;}
    is_scheduling.store(true, core::sync::atomic::Ordering::Relaxed);
    scheduler::schedule();
    is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
}

#[no_mangle]
pub unsafe extern "C" fn lower_aarch64_synchronous(context: &mut ExceptionContext){
    if context.esr_el1 == 1442840576 {
        // println!("Syscall happened {:?} ", *context);
        let syscall_type : Option<Syscalls> = Syscalls::from_u64(context.gpr.x[8]);
        // match syscall_type
        if syscall_type.is_none() {
            println!("WRONG SYSCALL {:?}", *context);
            return;
        }
        let syscall_type = syscall_type.unwrap();

        match syscall_type {
            Syscalls::Print => handle_print_syscall(context),
            Syscalls::NewTask => handle_new_task_syscall(context),
        }
    }
}

fn handle_print_syscall(context: &mut ExceptionContext){
    let ptr = context.gpr.x[0] as *const u8;
    let len = context.gpr.x[1] as usize;

    let data = unsafe { slice::from_raw_parts(ptr, len) };

    let string = from_utf8(data);

    if string.is_err(){
        println!("Print SYSCALL ERROR: NOT CORRECT UTF8 STRING PASSED");
        return;
    }
    let string = string.unwrap();
    
    print!("{}", string);
}


fn handle_new_task_syscall(context : &mut ExceptionContext){

    let start_function = unsafe { *(context.gpr.x[0] as *const () as *const extern "C" fn()) };
    let priority_difference = context.gpr.x[1] as u32;

    let curr_priority = scheduler::get_current_task_priority();
    let new_priority = if curr_priority > priority_difference {curr_priority - priority_difference} else {1};

    let task = scheduler::TaskContext::new(start_function, new_priority, true);
    task.start_task().unwrap();
}