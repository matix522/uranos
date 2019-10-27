use crate::gpio;

pub mod gicv2;
pub mod handlers;
pub mod timer;

pub enum Error {
    IrqNotEnabled,
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
pub fn disable_irqs() {
    unsafe {
        asm!("msr daifset, #2" : : : : "volatile");
    }
}
#[inline(always)]
pub fn enable_irqs() {
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
#[derive(Debug)]
pub enum InteruptError {
    IncorrectIrqNumber,
}
pub type InteruptResult = Result<(), InteruptError>;

pub trait InteruptController {
    fn init(&mut self) -> InteruptResult;

    fn enable_irq(&mut self, irq_number: usize) -> InteruptResult;
    fn disable_irq(&mut self, irq_number: usize) -> InteruptResult;

    fn connect_irq(
        &mut self,
        irq_number: usize,
        handler: Option<fn(data: &mut ExceptionContext)>,
    ) -> InteruptResult;
    fn disconnect_irq(&mut self, irq_number: usize) -> InteruptResult;
}

global_asm!(include_str!("vector_table.S"));
