pub mod handlers;

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

pub unsafe fn init_exceptions() {
    // Provided by interupt_context_saving.S.
    extern "C" {
        static mut __exception_vector_start: u64;
    }
    let addr: u64 = &__exception_vector_start as *const _ as u64;

    VBAR_EL1.set(addr);

    // Force VBAR update to complete before next instruction.
    barrier::isb(barrier::SY);
}

global_asm!(include_str!("interupts/interupt_context_saving.S"));


