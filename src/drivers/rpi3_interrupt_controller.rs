use crate::interupts::*;
use alloc::collections::BTreeMap;
use core::ops;
use register::{mmio::*, register_bitfields};
register_bitfields! {
    u32,
    /// The basic pending register shows which interrupt are pending
    IRQ_BASIC_PENDING [
        // If there is one or more pending interrupt of 1/2 group bit will be set
        ONE_OR_MORE_PENDING_1 OFFSET(8) NUMBITS(1) [],
        ONE_OR_MORE_PENDING_2 OFFSET(9) NUMBITS(1) [],

        ARM_TIMER_IRQ_PENDING OFFSET(0) NUMBITS(1) [],
        ARM_MAILBOX_IRQ_PENDING OFFSET(1) NUMBITS(1) [],
        ARM_GPU0_HALTED_PENDING OFFSET(4) NUMBITS(1) []
    ],
    IRQ_PENDING_2 [
        ARM_UART_IRQ_PENDING OFFSET(25) NUMBITS(1) []
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
    pub IRQ_BASIC_PENDING: ReadOnly<u32, IRQ_BASIC_PENDING::Register>,
    pub IRQ_PENDING_1: ReadOnly<u32>,
    pub IRQ_PENDING_2: ReadOnly<u32, IRQ_PENDING_2::Register>,
    pub FIQ_CONTROL: ReadWrite<u32>,
    pub ENABLE_IRQS_1: WriteOnly<u32>,
    pub ENABLE_IRQS_2: WriteOnly<u32, ENABLE_IRQs_2::Register>,
    pub ENABLE_BASIC_IRQS: WriteOnly<u32, ENABLE_BASIC_IRQs::Register>,
    pub DISABLE_IRQS_1: WriteOnly<u32>,
    pub DISABLE_IRQS_2: WriteOnly<u32, DISABLE_IRQs_2::Register>,
    pub DISABLE_BASIC_IRQS: WriteOnly<u32, DISABLE_BASIC_IRQs::Register>,
}
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum IRQType {
    ArmTimer,
    ArmMailbox,
    ArmGpioHalted,
    Uart,
}

pub struct Rpi3InterruptController {
    base_address: usize,
    irq_handler_map: BTreeMap<IRQType, fn(context: &mut ExceptionContext)>,
}

/// Deref to RegisterBlock
///
/// Allows writing
/// ```
/// self.ENABLE_IRQS_1.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*rpi3_interrupt_controller::ptr()).ENABLE_IRQS_1.read() }
/// ```
impl ops::Deref for Rpi3InterruptController {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl Rpi3InterruptController {
    pub const fn new(base_address: usize) -> Rpi3InterruptController {
        Rpi3InterruptController {
            base_address,
            irq_handler_map: BTreeMap::new(),
        }
    }
    /// Returns a pointer to the register block
    fn ptr(&self) -> *const RegisterBlock {
        self.base_address as *const _
    }
}

impl interrupt_controller::InterruptController for Rpi3InterruptController {
    type IRQNumberType = IRQType;

    fn enable_irq(&mut self, irq_number: Self::IRQNumberType) -> InterruptResult {
        match irq_number {
            IRQType::ArmGpioHalted => self
                .ENABLE_BASIC_IRQS
                .write(ENABLE_BASIC_IRQs::ARM_GPU0_HALTED_ENABLE::SET),
            IRQType::ArmMailbox => self
                .ENABLE_BASIC_IRQS
                .write(ENABLE_BASIC_IRQs::ARM_MAILBOX_IRQ_ENABLE::SET),
            IRQType::ArmTimer => self
                .ENABLE_BASIC_IRQS
                .write(ENABLE_BASIC_IRQs::ARM_TIMER_IRQ_ENABLE::SET),
            IRQType::Uart => self.ENABLE_IRQS_2.write(ENABLE_IRQs_2::UART_ENABLE::SET),
        }
        Ok(())
    }
    fn disable_irq(&mut self, irq_number: Self::IRQNumberType) -> InterruptResult {
        match irq_number {
            IRQType::ArmGpioHalted => self
                .DISABLE_BASIC_IRQS
                .write(DISABLE_BASIC_IRQs::ARM_GPU0_HALTED_DISABLE::SET),
            IRQType::ArmMailbox => self
                .DISABLE_BASIC_IRQS
                .write(DISABLE_BASIC_IRQs::ARM_MAILBOX_IRQ_DISABLE::SET),
            IRQType::ArmTimer => self
                .DISABLE_BASIC_IRQS
                .write(DISABLE_BASIC_IRQs::ARM_TIMER_IRQ_DISABLE::SET),
            IRQType::Uart => self.DISABLE_IRQS_2.write(DISABLE_IRQs_2::UART_DISABLE::SET),
        }
        Ok(())
    }
    fn is_pending_irq(&mut self, irq_number: Self::IRQNumberType) -> bool {
        match irq_number {
            IRQType::ArmGpioHalted => {
                self.IRQ_BASIC_PENDING
                    .read(IRQ_BASIC_PENDING::ARM_GPU0_HALTED_PENDING)
                    == 1
            }
            IRQType::ArmMailbox => {
                self.IRQ_BASIC_PENDING
                    .read(IRQ_BASIC_PENDING::ARM_MAILBOX_IRQ_PENDING)
                    == 1
            }
            IRQType::ArmTimer => {
                self.IRQ_BASIC_PENDING
                    .read(IRQ_BASIC_PENDING::ARM_TIMER_IRQ_PENDING)
                    == 1
            }
            IRQType::Uart => self.IRQ_PENDING_2.read(IRQ_PENDING_2::ARM_UART_IRQ_PENDING) == 1,
        }
    }
    fn connect_irq(
        &mut self,
        _irq_number: Self::IRQNumberType,
        _irq_descriptor: IRQDescriptor,
    ) -> InterruptResult {
        self.irq_handler_map
            .insert(_irq_number, _irq_descriptor.handler.unwrap());
        Ok(())
    }
}
