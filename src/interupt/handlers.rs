use super::*;
pub use crate::framebuffer::charbuffer::CharBuffer;
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
#[derive(FromPrimitive)]
enum SynchronousCause {
    SyscallFromEL0 = 0x5600_0000,
}
#[no_mangle]
pub unsafe extern "C" fn default_interupt_handler(context: &mut ExceptionContext, id: usize) -> ! {
    super::disable_irqs();
    println!(
        "Interupt Happened of ID-{}:\n  SP : {:#018x}\n {}",
        id, context as *mut ExceptionContext as u64, *context
    );
    println!("{:?}", boot::mode::ExceptionLevel::get_current());
    context.esr_el1 = 0;

    // let sp : u64;
    // unsafe {
    //     asm!("mov x8, sp" : "={x8}"(sp) : : : "volatile");
    // }
    // println!("SP: {}",sp);
    // unsafe { println!("*sp: {:x} *pc:{:x}", *(context.sp_el0 as *const u64), *(context.elr_el1 as *const u64) ) };
    // gpio::blink();
    // context.elr_el1 += 8;
    loop {}
}

static is_scheduling: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn end_scheduling() {
    unsafe {
        is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
    }
}

#[no_mangle]
pub unsafe extern "C" fn current_elx_irq(_context: &mut ExceptionContext) {
    // super::disable_irqs();
    Timer::interupt_after(Timer::get_frequency());
    Timer::enable();
    super::enable_irqs();
    if is_scheduling.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    is_scheduling.store(true, core::sync::atomic::Ordering::Relaxed);
    scheduler::schedule();
    is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
}

#[no_mangle]
pub unsafe extern "C" fn lower_aarch64_synchronous(context: &mut ExceptionContext) -> () {
    // println!("{}",*context);
    match SynchronousCause::from_u64(context.esr_el1) {
        Some(_) => {
            // println!("{}",*context);
            let syscall_type: Option<Syscalls> = Syscalls::from_u64(context.gpr.x[8]);

            if syscall_type.is_none() {
                println!(
                    "[Task Fault] Unsupported Syscall number '{}' detected.",
                    context.gpr.x[8]
                );
                return;
            }
            let syscall_type = syscall_type.unwrap();
            // println!("{}",*context);
            // println!("{} {:?}",context.gpr.x[8] ,syscall_type);

            match syscall_type {
                Syscalls::Print => handle_print_syscall(context),
                Syscalls::NewTask => handle_new_task_syscall(context),
                Syscalls::TerminateTask => handle_terminate_task_syscall(context),
            }
        }
        None => {
            let mut charbuffer = crate::framebuffer::charbuffer::CHARBUFFER.lock();
            if charbuffer.is_some() {
                let charbuffer = charbuffer.as_mut().unwrap();
                charbuffer.set_cursor((0, 0));
                charbuffer.background = (0, 0, 180, 255);
                charbuffer.puts("                                       *   * \n");
                charbuffer.puts(" THE TURQUOISE SCREEN OF ETERNAL DOOM!   |   \n");
                charbuffer.puts("                                      /\\/\\/\\/   \n");
                for i in 0..charbuffer.height - 10 {
                    charbuffer.putc('\n');
                }
                charbuffer.update();
            }
            println!(
                "[Task Fault]\n\tReason: Unknown code '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tStack:               '{:#018x}\n",
                context.esr_el1,
                context.elr_el1,
                context.far_el1,
                context.sp_el0
            );
            loop {}
        }
    }
}

fn handle_print_syscall(context: &mut ExceptionContext) {
    let ptr = context.gpr.x[0] as *const u8;
    let len = context.gpr.x[1] as usize;

    // println!("{:x} {}", ptr as u64, len);
    // println!("{}", *context);

    let data = unsafe { slice::from_raw_parts(ptr, len) };

    let string = from_utf8(data);

    if string.is_err() {
        println!(
            "[Syscall Fault (Write)] String provided doesen't apper to be correct UTF-8 string.
            \n\t -- Caused by: '{}'",
            string.err().unwrap()
        );
        return;
    }
    let string = string.unwrap();
    // println!("{}", string);
    let mut charbuffer = crate::framebuffer::charbuffer::CHARBUFFER.lock();
    if charbuffer.is_some() {
        charbuffer.as_mut().unwrap().puts(string);
    } else {
        print!("{}", string);
    }
}

fn handle_new_task_syscall(context: &mut ExceptionContext) {
    let start_function = unsafe { &*(&context.gpr.x[0] as *const u64 as *const extern "C" fn()) };

    let priority_difference = context.gpr.x[1] as u32;

    let curr_priority = 1; //scheduler::get_current_task_priority();
    let new_priority = if curr_priority > priority_difference {
        curr_priority - priority_difference
    } else {
        1
    };
    let task = scheduler::TaskContext::new(*start_function, new_priority, true).unwrap();
    is_scheduling.store(true, core::sync::atomic::Ordering::Relaxed);
    match task.start_task() {
        Ok(_) => {}
        Err(e) => {
            println!(
                "[Syscall Fault (New Task)] System was unable to create new task.
            \n\t -- Caused by: '{:?}'",
                e
            );
        }
    }
    is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
}
fn handle_terminate_task_syscall(context: &mut ExceptionContext) {
    is_scheduling.store(true, core::sync::atomic::Ordering::Relaxed);

    match scheduler::end_task(scheduler::get_current_task_id()) {
        Ok(_) => {}
        Err(e) => {
            println!(
                "[Kernel Fault] System was unable to terminate task.
            \n\t -- Caused by: '{:?}'",
                e
            );
            aarch64::halt();
        }
    }
    scheduler::schedule();
    is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
}
