pub mod handlers;
pub mod interrupt_controller;

use core::sync::atomic::{fence, Ordering};
use cortex_a::barrier;
use cortex_a::regs::*;

#[repr(C)]
pub struct ExceptionContext {
    /// General Purpose Registers.
    pub(super) gpr: [u64; 30],
    /// The link register, aka x30.
    pub(super) lr: u64,
    /// Exception link register. The program counter at the time the exception happened.
    pub(super) elr_el1: u64,
    /// Saved program status.
    pub(super) spsr_el1: u64,

    pub(super) esr_el1: u64,

    pub(super) far_el1: u64,

    pub(super) sp_el0: u64,
}
///
/// # Safety
/// User is required to make sure exception_vector_addr is address of correct interupt vector
pub unsafe fn init_exceptions(exception_vector_addr: usize) {
    VBAR_EL1.set(exception_vector_addr as u64);

    // Force VBAR update to complete before next instruction.
    barrier::isb(barrier::SY);
    fence(Ordering::SeqCst);
}

global_asm!(include_str!("interupts/interupt_context_saving.S"));

#[derive(Debug)]
pub enum InterruptError {
    IncorrectIrqNumber,
}
pub type InterruptResult = Result<(), InterruptError>;

impl core::convert::From<InterruptError> for &'static str {
    fn from(err: InterruptError) -> &'static str {
        match err {
            InterruptError::IncorrectIrqNumber => "IncorrectIrqNumber",
        }
    }
}

pub struct IRQDescriptor {
    pub name: &'static str,
    pub handler: Option<fn(context: &mut ExceptionContext)>,
}

#[inline(always)]
pub fn disable_irqs() {
    unsafe {
        llvm_asm!("msr daifset, #15" : : : : "volatile");
    }
}
#[inline(always)]
pub fn enable_irqs() {
    unsafe {
        llvm_asm!("msr daifclr, #15" : : : : "volatile");
    }
}
#[inline(always)]
pub fn set_vector_table_pointer(address: u64) {
    unsafe {
        llvm_asm!("msr vbar_el1, $0" : :  "r"(address) : : "volatile");
    }
}
