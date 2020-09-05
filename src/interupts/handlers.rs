use crate::drivers::arm_timer::ArmTimer;
use crate::drivers::traits::time::Timer;
use crate::interupts;
use crate::interupts::ExceptionContext;
use crate::scheduler;
use crate::syscall;
use crate::syscall::Syscalls;
pub use num_traits::FromPrimitive;

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

    let mut i = 0;
    unsafe {
        for elem in &context.gpr {
            crate::println!("GPR[{}]: {:#018x}", i, elem);
            i = i+1;
        }
        crate::println!("LR: {:#018x}", context.lr);
        crate::println!("ELR_EL1: {:#018x}", context.elr_el1);
    }
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
static COUNTER: AtomicI64 = AtomicI64::new(0);

#[no_mangle]
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    interupts::disable_irqs();

    let exception_type = (e.esr_el1 & (0b111111 << 26)) >> 26;
    if exception_type == 0b111100 {
        if COUNTER.load(Ordering::SeqCst) == 0 {
            COUNTER.fetch_add(1, Ordering::SeqCst);
            e.elr_el1 = e.gpr[0] | crate::KERNEL_OFFSET as u64;
        } else {
            scheduler::switch_task();
        }
    } else if exception_type == 0b010101 {
        let syscall_type = Syscalls::from_u64(e.gpr[8]).unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::Yield => scheduler::switch_task(),
            Syscalls::StartScheduling => scheduler::start(),
            Syscalls::Print => syscall::print::handle_print_syscall(e),
            Syscalls::FinishTask => scheduler::finish_current_task(),
            Syscalls::CreateTask => scheduler::handle_new_task_syscall(e.gpr[0] as usize),
        }
    } else {
        default_exception_handler(e, "current_elx_synchronous");
    }
    
    // crate::println!(
    //     "=========================SYNCHRONOUS\nesr_el1 '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tLinkRegister:        '{:#018x}\n\tStackPointer:        '{:#018x}\n\t SPSR: {:#064b}\n",
    //     e.esr_el1,
    //     e.elr_el1,
    //     e.far_el1,
    //     e.lr,
    //     e.sp,
    //     e.spsr_el1
    // );
    // let mut i = 0;
    // unsafe {
    //     for elem in &e.gpr {
    //         crate::println!("GPR[{}]: {:#018x}", i, elem);
    //         i = i+1;
    //     }
    //     crate::println!("LR: {:#018x}", e.lr);
    //     crate::println!("ELR_EL1: {:#018x}", e.elr_el1);
    // }
    interupts::enable_irqs();
}


static is_scheduling: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn end_scheduling() {
    unsafe {
        is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);
    }
    interupts::enable_irqs();
}


#[no_mangle]
unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    interupts::disable_irqs();

    let timer = ArmTimer {};
    timer.interupt_after(scheduler::get_time_quant());
    timer.enable();


    if is_scheduling.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    is_scheduling.store(true, core::sync::atomic::Ordering::Relaxed);
    scheduler::switch_task();
    is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);

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
    if exception_type == 0b111100 {
        if COUNTER.load(Ordering::SeqCst) == 0 {
            COUNTER.fetch_add(1, Ordering::SeqCst);
            e.elr_el1 = e.gpr[0] | crate::KERNEL_OFFSET as u64;
        } else {
            scheduler::switch_task();
        }
    } else if exception_type == 0b010101 {
        let syscall_type = Syscalls::from_u64(e.gpr[8]).unwrap_or_else(|| panic!("Unknown syscall type {}", e.gpr[8]));
        match syscall_type {
            Syscalls::Yield => scheduler::switch_task(),
            Syscalls::StartScheduling => scheduler::start(),
            Syscalls::Print => syscall::print::handle_print_syscall(e),
            Syscalls::FinishTask => scheduler::finish_current_task(),
            Syscalls::CreateTask => scheduler::handle_new_task_syscall(e.gpr[0] as usize),
        }
    } else {
        default_exception_handler(e, "current_elx_synchronous");
    }
    // crate::println!("{:#018x}'\n\tProgram location:    '{:#018x}'", e as *mut _ as usize, e.elr_el1);



    // crate::println!(
    //     "=========================SYNCHRONOUS\nesr_el1 '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tLinkRegister:        '{:#018x}\n\tStackPointer:        '{:#018x}\n\t SPSR: {:#064b}\n",
    //     e.esr_el1,
    //     e.elr_el1,
    //     e.far_el1,
    //     e.lr,
    //     e.sp,
    //     e.spsr_el1
    // );
    // let mut i = 0;
    // unsafe {
    //     for elem in &e.gpr {
    //         crate::println!("GPR[{}]: {:#018x}", i, elem);
    //         i = i+1;
    //     }
    //     crate::println!("LR: {:#018x}", e.lr);
    //     crate::println!("ELR_EL1: {:#018x}", e.elr_el1);
    // }
    interupts::enable_irqs();


}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    interupts::disable_irqs();

    let timer = ArmTimer {};
    timer.interupt_after(scheduler::get_time_quant());
    timer.enable();


    if is_scheduling.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    is_scheduling.store(true, core::sync::atomic::Ordering::Relaxed);
    scheduler::switch_task();
    is_scheduling.store(false, core::sync::atomic::Ordering::Relaxed);

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
unsafe extern "C" fn label_jakis(e: &mut ExceptionContext) {
    crate::println!(
        "esr_el1 '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tLinkRegister:        '{:#018x}\n\tStackPointer:        '{:#018x}\n\t SPSR: {:#064b}\n",
        e.esr_el1,
        e.elr_el1,
        e.far_el1,
        e.lr,
        e.sp,
        e.spsr_el1
    );
    let mut i = 0;
    unsafe {
        for elem in &e.gpr {
            crate::println!("GPR[{}]: {:#018x}", i, elem);
            i = i+1;
        }
        crate::println!("LR: {:#018x}", e.lr);
        crate::println!("ELR_EL1: {:#018x}", e.elr_el1);
    }
}



