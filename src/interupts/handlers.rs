use crate::drivers::arm_timer::ArmTimer;
use crate::drivers::traits::time::Timer;
use crate::interupts;
use crate::interupts::ExceptionContext;
use crate::scheduler;
use crate::syscall;
use crate::syscall::Syscalls;

use crate::config;
use core::convert::TryInto;

pub use num_traits::FromPrimitive;

const BRK_FLAG: u64 = 0b111100;
const SVC_FLAG: u64 = 0b010101;

fn handle_chcek_el(e: &mut ExceptionContext) {
    e.gpr[0] = match e.spsr_el1 & 0b1111 {
        0b0000 => 0,
        0b0101 => 1,
        _ => 3,
    };
}

fn default_exception_handler(context: &mut ExceptionContext, source: &str) {
    crate::println!(
        "[Task Fault]\n\tReason: Unknown code '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tLinkRegister:        '{:#018x}\n\tStackPointer:        '{:#018x}\n\t SPSR: {:#064b}\n",
        context.esr_el1,
        context.elr_el1,
        context.far_el1,
        context.lr,
        context.sp,
        context.spsr_el1
    );

    for (i, elem) in context.gpr.iter().enumerate() {
        crate::println!("GPR[{}]: {:#018x}", i, elem);
    }
    crate::println!("LR: {:#018x}", context.lr);
    crate::println!("ELR_EL1: {:#018x}", context.elr_el1);

    panic!("Unknown {} Exception type recived.", source);
}

//------------------------------------------------------------------------------
// Current, EL0
//------------------------------------------------------------------------------
#[no_mangle]
unsafe extern "C" fn current_el0_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_el0_synchronous");
}

#[no_mangle]
unsafe extern "C" fn current_el0_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_el0_irq");
}

#[no_mangle]
unsafe extern "C" fn current_el0_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_el0_serror");
}

//------------------------------------------------------------------------------
// Current, ELx
//------------------------------------------------------------------------------
use core::sync::atomic::*;

#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    interupts::disable_irqs();

    let exception_type = (e.esr_el1 & (0b111111 << 26)) >> 26;
    if exception_type == BRK_FLAG && !config::use_user_space() {
        config::set_use_user_space(true);
        e.elr_el1 = e.gpr[0] | crate::KERNEL_OFFSET as u64;
    } else if exception_type == BRK_FLAG {
        e.spsr_el1 = 0b0;
        e.elr_el1 = e.gpr[0] & (!crate::KERNEL_OFFSET) as u64;
    } else if exception_type == SVC_FLAG {
        let syscall_type = Syscalls::from_u64(e.gpr[8])
            .unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::Yield => scheduler::switch_task(),
            Syscalls::StartScheduling => scheduler::start(),
            Syscalls::Print => syscall::print::handle_print_syscall(e),
            Syscalls::FinishTask => scheduler::finish_current_task((e.gpr[0] as u32).try_into().unwrap_or_else(|_| scheduler::special_return_vals::WRONG_RETURN_VALUE_PASSED)),
            Syscalls::CreateTask => scheduler::handle_new_task_syscall(e),
            Syscalls::CheckEL => handle_chcek_el(e),
            Syscalls::GetAsyncSubmissionBuffer => {
                syscall::asynchronous::handle_get_submission_buffer::handle_get_submission_buffer(e)
            }
            Syscalls::GetAsyncCompletionBuffer => {
                syscall::asynchronous::handle_get_completion_buffer::handle_get_completion_buffer(e)
            }
            Syscalls::OpenFile => syscall::files::open::handle_open(e),
            Syscalls::CloseFile => syscall::files::close::handle_close(e),
            Syscalls::ReadFile => syscall::files::read::handle_read(e),
            Syscalls::SeekFile => syscall::files::seek::handle_seek(e),
            Syscalls::WriteFile => syscall::files::write::handle_write(e),
            Syscalls::GetPID => {
                e.gpr[0] = scheduler::get_current_task_pid() as u64;
            }
        }
    } else {
        default_exception_handler(e, "current_elx_synchronous");
    }

    interupts::enable_irqs();
}

static IS_SCHEDULING: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn end_scheduling() {
    IS_SCHEDULING.store(false, core::sync::atomic::Ordering::Relaxed);
    interupts::enable_irqs();
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq(_e: &mut ExceptionContext) {
    interupts::disable_irqs();

    let timer = ArmTimer {};
    timer.interupt_after(scheduler::get_time_quant());
    timer.enable();

    if IS_SCHEDULING.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    IS_SCHEDULING.store(true, core::sync::atomic::Ordering::Relaxed);
    scheduler::switch_task();
    IS_SCHEDULING.store(false, core::sync::atomic::Ordering::Relaxed);
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_elx_serror");
}

//------------------------------------------------------------------------------
// Lower, AArch64
//------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    interupts::disable_irqs();

    let exception_type = (e.esr_el1 & (0b111111 << 26)) >> 26;
    if exception_type == BRK_FLAG {
        e.elr_el1 = e.gpr[0] | crate::KERNEL_OFFSET as u64;
    } else if exception_type == SVC_FLAG {
        let syscall_type = Syscalls::from_u64(e.gpr[8])
            .unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::Yield => scheduler::switch_task(),
            Syscalls::StartScheduling => scheduler::start(),
            Syscalls::Print => syscall::print::handle_print_syscall(e),
            Syscalls::FinishTask => scheduler::finish_current_task((e.gpr[0] as u32).try_into().unwrap_or_else(|_| scheduler::special_return_vals::WRONG_RETURN_VALUE_PASSED)),
            Syscalls::CreateTask => scheduler::handle_new_task_syscall(e),
            Syscalls::CheckEL => handle_chcek_el(e),
            Syscalls::GetAsyncSubmissionBuffer => {
                syscall::asynchronous::handle_get_submission_buffer::handle_get_submission_buffer(e)
            }
            Syscalls::GetAsyncCompletionBuffer => {
                syscall::asynchronous::handle_get_completion_buffer::handle_get_completion_buffer(e)
            }
            Syscalls::OpenFile => syscall::files::open::handle_open(e),
            Syscalls::CloseFile => syscall::files::close::handle_close(e),
            Syscalls::ReadFile => syscall::files::read::handle_read(e),
            Syscalls::SeekFile => syscall::files::seek::handle_seek(e),
            Syscalls::WriteFile => syscall::files::write::handle_write(e),
            Syscalls::GetPID => {
                e.gpr[0] = scheduler::get_current_task_pid() as u64;
            }
        }
    } else {
        default_exception_handler(e, "lower_aarch64_synchronous");
    }

    interupts::enable_irqs();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(_e: &mut ExceptionContext) {
    interupts::disable_irqs();

    let timer = ArmTimer {};
    timer.interupt_after(scheduler::get_time_quant());
    timer.enable();

    if IS_SCHEDULING.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    IS_SCHEDULING.store(true, core::sync::atomic::Ordering::Relaxed);
    scheduler::switch_task();
    IS_SCHEDULING.store(false, core::sync::atomic::Ordering::Relaxed);
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch64_serror");
}

//------------------------------------------------------------------------------
// Lower, AArch32
//------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_synchronous");
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_irq");
}

#[no_mangle]
unsafe extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch32_serror");
}
