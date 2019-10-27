use crate::gpio;
use crate::println;
use register::{mmio::*, register_bitfields};

pub mod gicv2;
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
#[derive(Debug)]
pub struct GPR {
    x: [u64; 31],
}

#[repr(C)]
#[derive(Debug)]
pub struct ExceptionContext {
    // General Purpose Registers
    gpr: GPR,
    spsr_el1: u64,
    elr_el1: u64,
    esr_el1: u64,
}

#[inline(always)]
pub fn disable_IRQs() {
    unsafe {
        asm!("msr daifset, #2" : : : : "volatile");
    }
}
#[inline(always)]
pub fn enable_IRQs() {
    unsafe {
        asm!("msr daifclr, #2" : : : : "volatile");
    }
}
#[inline(always)]
pub fn set_vector_table_pointer(address: u64) {
    unsafe {
        asm!("msr vbar_el1, $0" : :  "r"(address) : : "volatile");
    }
}

pub enum InteruptError {
    IncorrectIrqNumber,
}
pub type InteruptResult = Result<(), InteruptError>;

trait InteruptController {
    fn init(&mut self) -> InteruptResult;

    fn enableIRQ(&mut self, irq_number: usize) -> InteruptResult;
    fn disableIRQ(&mut self, irq_number: usize) -> InteruptResult;

    fn connectIRQ(
        &mut self,
        irq_number: usize,
        handler: Option<&'static fn(data: &mut ExceptionContext)>,
    ) -> InteruptResult;
    fn disconnectIRQ(&mut self, irq_number: usize) -> InteruptResult;
}

global_asm!(include_str!("vector_table.S"));
