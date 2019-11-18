use super::*;
use crate::println;
use crate::scheduler;
use timer::ArmQemuTimer as Timer;
use core::sync::atomic::AtomicBool;

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
