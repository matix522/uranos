use crate::interupts::{IRQDescriptor, InterruptResult};

pub trait interruptController {
    type IRQNumberType;

    fn enable_IRQ(&self, irq_number: Self::IRQNumberType) -> InterruptResult;
    fn disable_IRQ(&self, irq_number: Self::IRQNumberType) -> InterruptResult;
    
    fn connect_irq(
        &self,
        irq_number: Self::IRQNumberType,
        irq_descriptor: IRQDescriptor,
    ) -> InterruptResult;

    
}