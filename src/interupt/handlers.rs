use super::*;
use crate::print;
use crate::println;
use timer::ArmQemuTimer as Timer;

#[no_mangle]
pub unsafe extern "C" fn default_interupt_handler(context: &mut ExceptionContext) {
    println!("Interupt Happened");
    gpio::blink();
}

static mut SECONDS: u64 = 0;


#[no_mangle]
pub unsafe extern "C" fn current_elx_irq(context: &mut ExceptionContext) {
    super::daif_set(2);
    Timer::disable();
    Timer::interupt_after(Timer::get_frequency());

    SECONDS += 1;
    let sec;
    if SECONDS == 1 {
        sec = "second";
    } else {
        sec = "seconds";
    }
    println!("\x1B[s\x1B[1;1H\x1B[38;5;204mTimer interupt happened \x1B[38;5;39m{} {}\x1B[38;5;204m after startup\x1B[0m\x1B[K", SECONDS, sec);
    print!("\x1B[u");
    //println!("Timer interupt happened {} {} after startup", SECONDS, sec);



    Timer::enable();
    super::daif_clr(2);
}
