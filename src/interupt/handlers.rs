use super::*;
use crate::interupt::ExceptionContext;
use crate::print;
use crate::println;
use crate::scheduler;
use crate::userspace::Syscalls;
use core::slice;
use core::str::*;
use core::sync::atomic::AtomicBool;
pub use num_traits::FromPrimitive;
use timer::ArmQemuTimer as Timer;

#[no_mangle]
pub unsafe extern "C" fn default_interupt_handler(context: &mut ExceptionContext, id: usize) {
    println!("Interupt Happened of ID-{}:  {:?}", id, *context);
    gpio::blink();
}

static mut is_scheduling: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn end_scheduling() {
    unsafe {
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
    if (is_scheduling.load(core::sync::atomic::Ordering::Relaxed)) {
        return;
    }
    is_scheduling.store(true, core::sync::atomic::Ordering::Relaxed);
    scheduler::schedule();
    is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
}

#[no_mangle]
pub unsafe extern "C" fn lower_aarch64_synchronous(context: &mut ExceptionContext) {
    if context.esr_el1 == 1442840576 {
        // println!("Syscall happened {:?} ", *context);
        let syscall_type: Option<Syscalls> = Syscalls::from_u64(context.gpr.x[8]);
        // match syscall_type
        if syscall_type.is_none() {
            println!("[Task Fault] Unsupported Syscall number '{}' detected ", context.gpr.x[8]);
            return;
        }
        let syscall_type = syscall_type.unwrap();

        match syscall_type {
            Syscalls::Print => handle_print_syscall(context),
        }
    }
}

fn handle_print_syscall(context: &mut ExceptionContext) {
    // println!("{:?}", context);
    let ptr = context.gpr.x[0] as *const u8;
    let len = context.gpr.x[1] as usize;
    // println!("{:x} len: {}", ptr as u64 ,len);
    let data = unsafe { slice::from_raw_parts(ptr, len) };

    let string = from_utf8(data);
    let string2 = unsafe { from_utf8_unchecked(data) };

    if string.is_err() {
        // println!("Print SYSCALL ERROR: {}, {}", string.err().unwrap(), string2);
        return;
    }
    let string = string.unwrap();

    let mut charbuffer = crate::framebuffer::charbuffer::CHARBUFFER.lock();
    charbuffer.as_mut().unwrap().puts(string);
    //print!("{}", string);
}
