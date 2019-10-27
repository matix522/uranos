use super::*;
use crate::println;
use crate::scheduler;
use timer::ArmQemuTimer as Timer;

#[no_mangle]
pub unsafe extern "C" fn default_interupt_handler(context: &mut ExceptionContext) {
    println!("Interupt Happened {:?}", *context);
    gpio::blink();
}

#[no_mangle]
pub unsafe extern "C" fn current_elx_irq(_context: &mut ExceptionContext) {
    println!("\nXDDDDDDDD");

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
    println!("freq: {}", Timer::get_frequency());
    println!("\nSCHEDULING TIME!!!");
    Timer::enable();
    super::enable_irqs();
    scheduler::schedule();
    // super::daif_set(2);
    // Timer::disable();

    println!("END OF INTERRUPT");
}
