pub mod gicv2;
pub mod handlers;
pub mod timer;

pub enum Error {
    IrqNotEnabled,
}

#[repr(C)]
#[derive(Debug)]
pub struct GPR {
    pub x: [u64; 30],
    pub lr: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct ExceptionContext {
    // General Purpose Registers
    pub gpr: GPR,
    pub spsr_el1: u64,
    pub elr_el1: u64,
    pub esr_el1: u64,
    pub sp_el0: u64,
    pub far_el1: u64,
}

impl core::fmt::Display for ExceptionContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let alternating = |x| -> _ {
            if x % 2 == 0 {
                "   "
            } else {
                "\n"
            }
        };
        writeln!(f, "SPSR_EL1 : {:#018x}", self.spsr_el1)?;
        writeln!(f, " ELR_EL1 : {:#018x}", self.elr_el1)?;
        writeln!(f, " ESR_EL1 : {:#018x}", self.esr_el1)?;
        writeln!(f, "  SP_EL0 : {:#018x}", self.sp_el0)?;
        writeln!(f, " FAR_EL1 : {:#018x}", self.far_el1)?;

        writeln!(f, "General purpose register:")?;
        for (i, reg) in self.gpr.x.iter().enumerate() {
            write!(f, "      x{: <2}: {: >#018x}{}", i, reg, alternating(i))?;
        }
        write!(f, "      lr : {:#018x}", self.gpr.lr)?;
        Ok(())
    }
}

#[inline(always)]
pub fn disable_irqs() {
    unsafe {
        asm!("msr daifset, #15" : : : : "volatile");
    }
}
#[inline(always)]
pub fn enable_irqs() {
    unsafe {
        asm!("msr daifclr, #15" : : : : "volatile");
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
