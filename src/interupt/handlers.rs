use super::*;
use crate::print;
use crate::println;
use crate::scheduler;
use timer::ArmQemuTimer as Timer;

#[no_mangle]
pub unsafe extern "C" fn default_interupt_handler(context: &mut ExceptionContext) {
    println!("Interupt Happened {:?}", *context);
    gpio::blink();
}

static mut SECONDS: u64 = 0;

#[no_mangle]
pub unsafe extern "C" fn current_elx_irq(context: &mut ExceptionContext) {
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
    println!("\nSCHEDULING TIME!!!");
    Timer::enable();
    super::daif_clr(2);
    scheduler::schedule();
    // super::daif_set(2);
    // Timer::disable();

    println!("END OF INTERRUPT");
}
