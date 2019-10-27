use super::ExceptionContext;
use super::InteruptController;
use register::{mmio::*, register_bitfields};

const GIC_CORE_BASE_ADDRESS: usize = 0xff84_1000;
const GIC_COMMON_BASE_ADDRESS: usize = 0xff84_2000;
const GIC_END: usize = 0xff84_7fff;

const IRQ_LINES: usize = 256;

const GICD_IPRIORITYR_DEFAULT: u32 = 0xA0;
const GICD_ITARGETSR_CORE0: u32 = 0x00;

const GICC_CTLR_ENABLE: u32 = 1 << 0;

const GICC_PMR_PRIORITY: u32 = 0xF0 << 0;

pub struct RegisterBlock {
    GICD_CTLR: WriteOnly<u32>,                 // 0x000
    reserved_0: [u32; 0x19],                   // 0x004
    GICD_IGROUPR0: [WriteOnly<u32>; 0x20],     // 0x080
    GICD_ISENABLER0: [WriteOnly<u32>; 0x20],   // 0x100
    GICD_ICENABLER0: [WriteOnly<u32>; 0x20],   // 0x180
    GICD_ISPENDR0: [WriteOnly<u32>; 0x20],     // 0x200
    GICD_ICPENDR0: [WriteOnly<u32>; 0x20],     // 0x280
    GICD_ISACTIVER0: [WriteOnly<u32>; 0x20],   // 0x300
    GICD_ICACTIVER0: [WriteOnly<u32>; 0x20],   // 0x380
    GICD_IPRIORITYR0: [WriteOnly<u32>; 0x100], // 0x400
    GICD_ITARGETSR0: [WriteOnly<u32>; 0x100],  // 0x800
    GICD_ICFGR0: [WriteOnly<u32>; 0x40],       // 0xc00
    reserved_1: [WriteOnly<u32>; 0x2c0],       // 0xc40
    GICD_SGIR: [WriteOnly<u32>; 0x100],        // 0xf00
    GICC_CTLR: WriteOnly<u32>,                 // 0x1000
    GICC_PMR: WriteOnly<u32>,                  // 0x1004
    reserved_2: u32,                           // 0x1008
    GICC_IAR: WriteOnly<u32>,                  // 0x100c
    GICC_EOIR: WriteOnly<u32>,                 // 0x1010
}

pub struct GICv2 {
    irq_handlers: [Option<&'static fn(&mut super::ExceptionContext)>; IRQ_LINES],
}

impl GICv2 {
    pub fn new() -> Self {
        GICv2 {
            irq_handlers: [None; IRQ_LINES],
        }
    }
    fn ptr(&self) -> *mut RegisterBlock {
        GIC_CORE_BASE_ADDRESS as *mut RegisterBlock
    }
}
impl core::ops::Deref for GICv2 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*(self.ptr()) }
    }
}
use super::InteruptError;
use super::InteruptResult;

impl InteruptController for GICv2 {
    fn init(&mut self) -> Result<(), super::InteruptError> {
        self.GICD_CTLR.set(0);

        // disable, acknowledge and deactivate all interrupts
        for i in 0..IRQ_LINES / 32 {
            self.GICD_ICENABLER0[i].set(!0);
            self.GICD_ICPENDR0[i].set(!0);
            self.GICD_ICACTIVER0[i].set(!0);
        }

        // direct all interrupts to core 0 with default priority
        for i in 0..IRQ_LINES / 4 {
            self.GICD_IPRIORITYR0[i].set(
                GICD_IPRIORITYR_DEFAULT
                    | GICD_IPRIORITYR_DEFAULT << 8
                    | GICD_IPRIORITYR_DEFAULT << 16
                    | GICD_IPRIORITYR_DEFAULT << 24,
            );

            self.GICD_ITARGETSR0[i].set(
                GICD_ITARGETSR_CORE0
                    | GICD_ITARGETSR_CORE0 << 8
                    | GICD_ITARGETSR_CORE0 << 16
                    | GICD_ITARGETSR_CORE0 << 24,
            );
        }

        // set all interrupts to level triggered
        for i in 0..IRQ_LINES / 16 {
            self.GICD_ICFGR0[i].set(0);
        }

        self.GICD_CTLR.set(1); // enable controler

        // initialize core 0 CPU interface:

        self.GICC_PMR.set(GICC_PMR_PRIORITY);
        self.GICC_CTLR.set(GICC_CTLR_ENABLE);

        super::enable_IRQs();

        Ok(())
    }
    fn enableIRQ(&mut self, irq_number: usize) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        self.GICD_ISENABLER0[irq_number / 32].set(1 << (irq_number as u32 % 32));
        Ok(())
    }
    fn disableIRQ(&mut self, irq_number: usize) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        self.GICD_ICENABLER0[irq_number / 32].set(1 << (irq_number as u32 % 32));
        Ok(())
    }
    fn connectIRQ(
        &mut self,
        irq_number: usize,
        handler: Option<&'static fn(data: &mut ExceptionContext)>,
    ) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        self.irq_handlers[irq_number as usize] = handler;
        self.enableIRQ(irq_number)
    }
    fn disconnectIRQ(&mut self, irq_number: usize) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        self.disableIRQ(irq_number)?;
        self.irq_handlers[irq_number] = None;
        Ok(())
    }

    // void CInterruptSystem::ConnectIRQ (unsigned nIRQ, TIRQHandler *pHandler, void *pParam)
    // {
    // 	assert (nIRQ < IRQ_LINES);
    // 	assert (m_apIRQHandler[nIRQ] == 0);

    // 	m_apIRQHandler[nIRQ] = pHandler;
    // 	m_pParam[nIRQ] = pParam;

    // 	EnableIRQ (nIRQ);
    // }

    // void CInterruptSystem::DisconnectIRQ (unsigned nIRQ)
    // {
    // 	assert (nIRQ < IRQ_LINES);
    // 	assert (m_apIRQHandler[nIRQ] != 0);

    // 	DisableIRQ (nIRQ);

    // 	m_apIRQHandler[nIRQ] = 0;
    // 	m_pParam[nIRQ] = 0;
    // }

    // void CInterruptSystem::EnableIRQ (unsigned nIRQ)
    // {
    // 	assert (nIRQ < IRQ_LINES);

    // 	write32 (GICD_ISENABLER0 + 4 * (nIRQ / 32), 1 << (nIRQ % 32));
    // }

    // void CInterruptSystem::DisableIRQ (unsigned nIRQ)
    // {
    // 	assert (nIRQ < IRQ_LINES);

    // 	write32 (GICD_ICENABLER0 + 4 * (nIRQ / 32), 1 << (nIRQ % 32));
    // }
}
