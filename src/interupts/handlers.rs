use crate::interupts::ExceptionContext;
use crate::scheduler;

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
unsafe extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) -> &ExceptionContext {
    let exception_type = (e.esr_el1 & (0b111111 << 26)) >> 26;
    if exception_type == 0b111100 {
        if COUNTER.load(Ordering::SeqCst) == 0 {
            COUNTER.fetch_add(1, Ordering::SeqCst);

            e.elr_el1 = e.gpr[0] | crate::KERNEL_OFFSET as u64;
        } else {
            let ec = scheduler::switch_task(e);
            return ec;
        }
    } else if exception_type == 0b010101 {
        match (e.gpr[8]) {
            0 => {
                return scheduler::switch_task(e);
            }
            _ => {
                return scheduler::start();
            }
        }
    } else {
        default_exception_handler(e, "current_elx_synchronous");
    }

    e
}

#[no_mangle]
unsafe extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_elx_irq");
}

#[no_mangle]
unsafe extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_exception_handler(e, "current_elx_serror");
}

//------------------------------------------------------------------------------
// Lower, AArch64
//------------------------------------------------------------------------------

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) -> &mut ExceptionContext {
    let exception_type = (e.esr_el1 & (0b111111 << 26)) >> 26;
    if exception_type == 0b111100 {
        e.elr_el1 = e.gpr[0] | crate::KERNEL_OFFSET as u64;
    } else if exception_type == 0b010101 {
        if COUNTER.load(Ordering::SeqCst) == 0 {
            COUNTER.fetch_add(1, Ordering::SeqCst);
            return scheduler::sample_change_task(e, true);
        } else {
            return scheduler::sample_change_task(e, false);
        }
    } else {
        default_exception_handler(e, "lower_aarch64_synchronous");
    }

    e
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    default_exception_handler(e, "lower_aarch64_irq");
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
