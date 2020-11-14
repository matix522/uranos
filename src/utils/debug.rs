use crate::drivers::rpi3_interrupt_controller::RegisterBlock;
use crate::interupts::ExceptionContext;
use crate::println;

pub fn print_register_address(block: &RegisterBlock) {
    println!(
        "IRQ_BASIC_PEND {:x}",
        &block.IRQ_BASIC_PENDING as *const _ as u64
    );
    println!(
        "IRQ_PENDING_1  {:x}",
        &block.IRQ_PENDING_1 as *const _ as u64
    );
    println!(
        "IRQ_PENDING_2  {:x}",
        &block.IRQ_PENDING_2 as *const _ as u64
    );
    println!("FIQ_CONTROL as {:x}", &block.FIQ_CONTROL as *const _ as u64);
    println!(
        "ENABLE_IRQS_1  {:x}",
        &block.ENABLE_IRQS_1 as *const _ as u64
    );
    println!(
        "ENABLE_IRQS_2  {:x}",
        &block.ENABLE_IRQS_2 as *const _ as u64
    );
    println!(
        "ENABLE_BASIC_I {:x}",
        &block.ENABLE_BASIC_IRQS as *const _ as u64
    );
    println!(
        "DISABLE_IRQS_1 {:x}",
        &block.DISABLE_IRQS_1 as *const _ as u64
    );
    println!(
        "DISABLE_IRQS_2 {:x}",
        &block.DISABLE_IRQS_2 as *const _ as u64
    );
    println!(
        "DISABLE_BASIC_ {:x}",
        &block.DISABLE_BASIC_IRQS as *const _ as u64
    );
}

#[no_mangle]
pub extern "C" fn debug_exception_context(e: &mut ExceptionContext) {
    crate::println!(
        "esr_el1 '{:#018x}'\n\tProgram location:    '{:#018x}'\n\tAddress:             '{:#018x}'\n\tLinkRegister:        '{:#018x}\n\tStackPointer:        '{:#018x}\n\t SPSR: {:#064b}\n",
        e.esr_el1,
        e.elr_el1,
        e.far_el1,
        e.lr,
        e.sp_el0,
        e.spsr_el1
    );
    for (i, elem) in e.gpr.iter().enumerate() {
        crate::println!("GPR[{}]: {:#018x}", i, elem);
    }
    crate::println!("LR: {:#018x}", e.lr);
    crate::println!("ELR_EL1: {:#018x}", e.elr_el1);
}
