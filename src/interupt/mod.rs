use crate::gpio;
use crate::println;
use register::{mmio::*, register_bitfields};

pub mod handlers;
pub mod timer;
pub enum Error {
    IrqNotEnabled,
}
register_bitfields! {
    u64,
    DAIF [
        Debug OFFSET(9) NUMBITS(1) [],
        Abort OFFSET(8) NUMBITS(1) [],
        IRQ   OFFSET(7) NUMBITS(1) [],
        FIQ   OFFSET(6) NUMBITS(1) []
    ]
}

#[repr(C)]
pub struct GPR {
    x: [u64; 31],
}

#[repr(C)]
pub struct ExceptionContext {
    // General Purpose Registers
    gpr: GPR,
    spsr_el1: u64,
    elr_el1: u64,
    esr_el1: u64,
}

/// TODO DAIF TYPE
pub fn daif_set(daif: u64) {
    unsafe {
        asm!("msr daifset, #2" : : : : "volatile");
    }
}
pub fn daif_clr(daif: u64) {
    unsafe {
        asm!("msr daifclr, #2" : : : : "volatile");
    }
}
pub fn set_vector_table_pointer(address: u64) {
    unsafe {
        asm!("msr vbar_el1, $0" : :  "r"(address) : : "volatile");
    }
}

global_asm!(include_str!("vector_table.S"));
