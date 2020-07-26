use crate::drivers::rpi3_interrupt_controller::RegisterBlock;
use crate::println;

pub fn printRegisterAddress(block: &RegisterBlock) {
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
