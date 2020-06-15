
use register::{mmio::*, register_bitfields};
use core::ops;
use crate::interupts::*;

register_bitfields! {
    u32,
    /// The basic pending register shows which interrupt are pending
    IRQ_BASIC_PENDING [
        // If there is one or more pending interrupt of 1/2 group bit will be set
        ONE_OR_MORE_PENDING_1 OFFSET(8) NUMBITS(1) [],
        ONE_OR_MORE_PENDING_2 OFFSET(9) NUMBITS(1) [],

        ARM_UART_IRQ_PENDING OFFSET(19) NUMBITS(1) [],
        ARM_TIMER_IRQ_PENDING OFFSET(0) NUMBITS(1) [],
        ARM_MAILBOX_IRQ_PENDING OFFSET(1) NUMBITS(1) [],
        ARM_GPU0_HALTED_PENDING OFFSET(4) NUMBITS(1) []
    ],
    ENABLE_IRQs_2 [
        UART_ENABLE OFFSET(25) NUMBITS(1) []
    ],
    ENABLE_BASIC_IRQs [
        ARM_TIMER_IRQ_ENABLE OFFSET(0) NUMBITS(1) [],
        ARM_MAILBOX_IRQ_ENABLE OFFSET(1) NUMBITS(1) [],
        ARM_GPU0_HALTED_ENABLE OFFSET(4) NUMBITS(1) []
    ],
    DISABLE_IRQs_2 [
        UART_DISABLE OFFSET(25) NUMBITS(1) []
    ],
    DISABLE_BASIC_IRQs [
        ARM_TIMER_IRQ_DISABLE OFFSET(0) NUMBITS(1) [],
        ARM_MAILBOX_IRQ_DISABLE OFFSET(1) NUMBITS(1) [],
        ARM_GPU0_HALTED_DISABLE OFFSET(4) NUMBITS(1) []
    ]
}


#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    IRQ_BASIC_PENDING: ReadOnly<u32, IRQ_BASIC_PENDING::Register>,                                          
    IRQ_PENDING_1: ReadOnly<u32>,                                         
    IRQ_PENDING_2: ReadOnly<u32>,                                         
    FIQ_CONTROL: ReadWrite<u32>,                                           
    ENABLE_IRQS_1: WriteOnly<u32>,                                         
    ENABLE_IRQS_2: WriteOnly<u32, ENABLE_IRQs_2::Register>,                                         
    ENABLE_BASIC_IRQS: WriteOnly<u32, ENABLE_BASIC_IRQs::Register>,                                       
    DISABLE_IRQS_1: WriteOnly<u32>,                                        
    DISABLE_IRQS_2: WriteOnly<u32, DISABLE_IRQs_2::Register>,                                        
    DISABLE_BASIC_IRQS: WriteOnly<u32, DISABLE_BASIC_IRQs::Register>,                                       
}


pub enum IRQType{
    ARM_TIMER,
    ARM_MAILBOX,
    ARM_GPIO_HALTED,
    UART,
}

pub struct Rpi3InterruptController{
    base_address: usize,
}

/// Deref to RegisterBlock
///
/// Allows writing
/// ```
/// self.ENABLE_IRQS_1.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*Rpi3InterruptController::ptr()).ENABLE_IRQS_1.read() }
/// ```
impl ops::Deref for Rpi3InterruptController {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl Rpi3InterruptController{
    pub const fn new(base_address: usize) -> Rpi3InterruptController{
        Rpi3InterruptController {base_address}
    }
    /// Returns a pointer to the register block
    fn ptr(&self) -> *const RegisterBlock {
        self.base_address as *const _
    }
}

impl interruptController::interruptController for Rpi3InterruptController{
    type IRQNumberType = IRQType;

    fn enable_IRQ(&mut self, irq_number: Self::IRQNumberType) -> InterruptResult{
        match irq_number{
            IRQType::ARM_GPIO_HALTED => Self.ENABLE_BASIC_IRQS.modify(ENABLE_BASIC_IRQs::ARM_GPU0_HALTED_ENABLE::SET),
            IRQType::ARM_MAILBOX     => Self.ENABLE_BASIC_IRQS.modify(ENABLE_BASIC_IRQs::ARM_MAILBOX_IRQ_ENABLE::SET),
            IRQType::ARM_TIMER       => Self.ENABLE_BASIC_IRQS.modify(ENABLE_BASIC_IRQs::ARM_TIMER_IRQ_ENABLE::SET),
            IRQType::UART            => Self.ENABLE_IRQS_2.modify(ENABLE_IRQs_2::UART_ENABLE::SET),
        }
        Ok(())
    }
    fn disable_IRQ(&mut self, irq_number: Self::IRQNumberType) -> InterruptResult{
        match irq_number{
            IRQType::ARM_GPIO_HALTED => Self.DISABLE_BASIC_IRQS.modify(DISABLE_BASIC_IRQs::ARM_GPU0_HALTED_DISABLE::SET),
            IRQType::ARM_MAILBOX     => Self.DISABLE_BASIC_IRQS.modify(DISABLE_BASIC_IRQs::ARM_MAILBOX_IRQ_DISABLE::SET),
            IRQType::ARM_TIMER       => Self.DISABLE_BASIC_IRQS.modify(DISABLE_BASIC_IRQs::ARM_TIMER_IRQ_DISABLE::SET),
            IRQType::UART            => Self.DISABLE_IRQS_2.modify(DISABLE_IRQs_2::UART_DISABLE::SET),
        }
        Ok(())
    }
    fn connect_irq(&self, irq_number: Self::IRQNumberType, irq_descriptor: IRQDescriptor) -> InterruptResult{
        Ok(())
    }
}