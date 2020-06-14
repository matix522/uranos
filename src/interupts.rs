pub mod handlers;

use core::sync::atomic::{fence, Ordering};
use cortex_a::barrier;
use cortex_a::regs::*;

#[repr(C)]
struct ExceptionContext {
    /// General Purpose Registers.
    gpr: [u64; 30],
    /// The link register, aka x30.
    lr: u64,
    /// Exception link register. The program counter at the time the exception happened.
    elr_el1: u64,
    /// Saved program status.
    spsr_el1: u64,
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
