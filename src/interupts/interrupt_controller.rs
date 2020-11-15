use crate::interupts::{IRQDescriptor, InterruptResult};

pub trait InterruptController {
    type IRQNumberType;

    fn enable_irq(&mut self, irq_number: Self::IRQNumberType) -> InterruptResult;
    fn disable_irq(&mut self, irq_number: Self::IRQNumberType) -> InterruptResult;
    fn is_pending_irq(&mut self, irq_number: Self::IRQNumberType) -> bool;

    fn connect_irq(
        &mut self,
        irq_number: Self::IRQNumberType,
        irq_descriptor: IRQDescriptor,
    ) -> InterruptResult;
}
