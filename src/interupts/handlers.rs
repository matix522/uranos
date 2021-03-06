use crate::drivers::arm_timer::ArmTimer;
use crate::drivers::traits::time::Timer;
use crate::interupts;
use crate::interupts::ExceptionContext;
use crate::scheduler;
use crate::syscall;
use crate::syscall::Syscalls;
use core::sync::atomic::*;

use crate::config;

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
        "[Task Fault]\n\tReason: Unknown code '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tLinkRegister:        '{:#018x}\n\tSP_EL0:              '{:#018x}\n\tSP_EL1:              '{:#018x}\n\tSP_EL1 LOWER:        '{:#018x}\n\t SPSR: {:#064b}\n",
        context.esr_el1,
        context.elr_el1,
        context.far_el1,
        context.lr,
        context.sp_el0,
        context as *const _ as u64,
        context as *const _ as u64 + core::mem::size_of::<ExceptionContext>() as u64,
        context.spsr_el1
    );

    for (i, elem) in context.gpr.iter().enumerate() {
        crate::println!("GPR[{}]: {:#018x}", i, elem);
    }
    crate::println!("LR: {:#018x}", context.lr);
    crate::println!("ELR_EL1: {:#018x}", context.elr_el1);

    // unsafe{
    //     let ptr = (context as *const ExceptionContext ).add(1);
    //     if (ptr as u64 % 0x1_0000 < 0x8000){
    //         crate::println!("SECOND PTR {:x}", ptr as u64);
    //         let context = & *ptr;
    //         crate::println!(
    //             "[Second]\n\tReason: Unknown code '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tLinkRegister:        '{:#018x}\n\tSP_EL0:              '{:#018x}\n\tSP_EL1:              '{:#018x}\n\tSP_EL1 LOWER:        '{:#018x}\n\t SPSR: {:#064b}\n",
    //             context.esr_el1,
    //             context.elr_el1,
    //             context.far_el1,
    //             context.lr,
    //             context.sp_el0,
    //             context as *const _ as u64,
    //             context as *const _ as u64 + core::mem::size_of::<ExceptionContext>() as u64,
    //             context.spsr_el1
    //         );

    //     }
    // }

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

#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    interupts::disable_irqs();
    // crate::utils::debug::debug_exception_context(e);

    let exception_type = (e.esr_el1 & (0b111111 << 26)) >> 26;

    if exception_type == SVC_FLAG {
        let syscall_type = Syscalls::from_u64(e.gpr[8])
            .unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::CheckEL => {}
            _ => crate::io::input_to_buffer(),
        }
    } else {
        crate::io::input_to_buffer();
    }

    if exception_type == BRK_FLAG && !config::use_user_space() {
        config::set_use_user_space(true);
        e.elr_el1 = e.gpr[2] | crate::KERNEL_OFFSET as u64;
    } else if exception_type == BRK_FLAG {
        e.spsr_el1 = 0b0;
        e.elr_el1 = e.gpr[2] & (!crate::KERNEL_OFFSET) as u64;
    } else if exception_type == SVC_FLAG {
        let syscall_type = Syscalls::from_u64(e.gpr[8])
            .unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::Yield => scheduler::switch_task(),
            Syscalls::StartScheduling => scheduler::start(),
            Syscalls::Print => syscall::print::handle_print_syscall(e),
            Syscalls::FinishTask => scheduler::finish_current_task(e.gpr[0] as u32),
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
            Syscalls::ReadFile => syscall::files::read::handle_read_syscall(e),
            Syscalls::SeekFile => syscall::files::seek::handle_seek(e),
            Syscalls::WriteFile => syscall::files::write::handle_write_syscall(e),
            Syscalls::CreateFile => syscall::files::create::handle_create(e),
            Syscalls::DeleteFile => syscall::files::delete::handle_delete(e),
            Syscalls::GetPID => {
                e.gpr[0] = scheduler::get_current_task_pid() as u64;
            }
            Syscalls::GetChildReturnValue => {
                e.gpr[0] = match scheduler::get_child_task_return_val(e.gpr[0] as usize) {
                    Some(value) => value as u64,
                    None => crate::utils::ONLY_MSB_OF_USIZE as u64,
                }
            }
            Syscalls::SetPipeReadOnPID => syscall::files::handle_set_pipe_read_on_pid(e),
        }
    } else {
        default_exception_handler(e, "current_elx_synchronous");
    }

    // interupts::enable_irqs();
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
    crate::io::input_to_buffer();

    crate::println!(
        "STACK IRQ PRE    {:#018x}",
        _e as *const ExceptionContext as u64
    );
    crate::println!("elr_el1 IRQ PRE  {:#018x}", _e.elr_el1);

    let timer = ArmTimer {};
    timer.interupt_after(scheduler::get_time_quant());
    timer.enable();

    if IS_SCHEDULING.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    IS_SCHEDULING.store(true, core::sync::atomic::Ordering::Relaxed);
    crate::syscall::asynchronous::handle_async_syscalls::handle_async_syscalls();

    scheduler::switch_task();
    IS_SCHEDULING.store(false, core::sync::atomic::Ordering::Relaxed);
    crate::println!(
        "STACK IRQ POST    {:#018x}",
        _e as *const ExceptionContext as u64
    );
    crate::println!("elr_el1 IRQ POST  {:#018x}", _e.elr_el1);
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

    // crate::print!("l");

    let exception_type = (e.esr_el1 & (0b111111 << 26)) >> 26;

    if exception_type == SVC_FLAG {
        let syscall_type = Syscalls::from_u64(e.gpr[8])
            .unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::CheckEL => {}
            _ => crate::io::input_to_buffer(),
        }
    } else {
        crate::io::input_to_buffer();
    }

    if exception_type == BRK_FLAG {
        e.elr_el1 = e.gpr[2] | crate::KERNEL_OFFSET as u64;
    } else if exception_type == SVC_FLAG {
        let syscall_type = Syscalls::from_u64(e.gpr[8])
            .unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::Yield => scheduler::switch_task(),
            Syscalls::StartScheduling => scheduler::start(),
            Syscalls::Print => syscall::print::handle_print_syscall(e),
            Syscalls::FinishTask => scheduler::finish_current_task(e.gpr[0] as u32),
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
            Syscalls::ReadFile => syscall::files::read::handle_read_syscall(e),
            Syscalls::SeekFile => syscall::files::seek::handle_seek(e),
            Syscalls::WriteFile => syscall::files::write::handle_write_syscall(e),
            Syscalls::CreateFile => syscall::files::create::handle_create(e),
            Syscalls::DeleteFile => syscall::files::delete::handle_delete(e),
            Syscalls::GetPID => {
                e.gpr[0] = scheduler::get_current_task_pid() as u64;
            }
            Syscalls::GetChildReturnValue => {
                e.gpr[0] = match scheduler::get_child_task_return_val(e.gpr[0] as usize) {
                    Some(value) => value as u64,
                    None => crate::utils::ONLY_MSB_OF_USIZE as u64,
                }
            }
            Syscalls::SetPipeReadOnPID => syscall::files::handle_set_pipe_read_on_pid(e),
        }
    } else {
        default_exception_handler(e, "lower_aarch64_synchronous");
    }

    // interupts::enable_irqs();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(_e: &mut ExceptionContext) {
    interupts::disable_irqs();

    crate::io::input_to_buffer();
    use crate::drivers::rpi3_interrupt_controller::IRQType;
    use crate::interupts::interrupt_controller::InterruptController;

    let mut controler = crate::drivers::INTERRUPT_CONTROLLER.lock();
    if controler.is_pending_irq(IRQType::Uart) {
        crate::eprintln!("UART");
        return;
    }

    let timer = ArmTimer {};
    timer.interupt_after(scheduler::get_time_quant());
    timer.enable();

    if IS_SCHEDULING.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    IS_SCHEDULING.store(true, core::sync::atomic::Ordering::Relaxed);

    crate::syscall::asynchronous::handle_async_syscalls::handle_async_syscalls();

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

#[no_mangle]
pub fn uart_fn(e: &mut ExceptionContext) {
    crate::eprintln!("HELLO UART");
    default_exception_handler(e, "lower_aarch32_serror");
}
