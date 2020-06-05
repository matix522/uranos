use super::ExceptionContext;
use super::InteruptController;
use register::mmio::*;
use register::register_structs;
const GIC_CORE_BASE_ADDRESS: usize = 0xff84_1000;
#[allow(unused)]
const GIC_COMMON_BASE_ADDRESS: usize = 0xff84_2000;
#[allow(unused)]
const GIC_END: usize = 0xff84_7fff;

const IRQ_LINES: usize = 256;

const GICD_IPRIORITYR_DEFAULT: u32 = 0xA0;
const GICD_ITARGETSR_CORE0: u32 = 0x01;

const GICC_CTLR_ENABLE: u32 = 1;

const GICC_PMR_PRIORITY: u32 = 0xF0;

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x000 => GICD_CTLR: WriteOnly<u32>),
        (0x004 => __reserved_0),
        (0x080 => GICD_IGROUPR0: [WriteOnly<u32>; 0x20]),
        (0x100 => GICD_ISENABLER0: [WriteOnly<u32>; 0x20]),
        (0x180 => GICD_ICENABLER0: [WriteOnly<u32>; 0x20]),
        (0x200 => GICD_ISPENDR0: [WriteOnly<u32>; 0x20]),
        (0x280 => GICD_ICPENDR0: [WriteOnly<u32>; 0x20]),
        (0x300 => GICD_ISACTIVER0: [WriteOnly<u32>; 0x20]),
        (0x380 => GICD_ICACTIVER0: [WriteOnly<u32>; 0x20]),
        (0x400 => GICD_IPRIORITYR0: [WriteOnly<u32>; 0x100]),
        (0x800 => GICD_ITARGETSR0: [WriteOnly<u32>; 0x100]),
        (0xc00 => GICD_ICFGR0: [WriteOnly<u32>; 0x40]),
        (0xd00 => __reserved_1),
        (0xf00 => GICD_SGIR: [WriteOnly<u32>; 0x40]),
        (0x1000 => GICC_CTLR: WriteOnly<u32>),
        (0x1004 => GICC_PMR: WriteOnly<u32>),
        (0x1008 => __reserved_2),
        (0x100c => GICC_IAR: WriteOnly<u32>),
        (0x1010 => GICC_EOIR: WriteOnly<u32>),
        (0x1014 => @END),
    }
}

// #[allow(non_snake_case)]
// #[repr(C)]
// pub struct RegisterBlock {
//     GICD_CTLR: WriteOnly<u32>,                 // 0x000
//     __reserved_0: [u32; 0x1f],                 // 0x004
//     GICD_IGROUPR0: [WriteOnly<u32>; 0x20],     // 0x080
//     GICD_ISENABLER0: [WriteOnly<u32>; 0x20],   // 0x100
//     GICD_ICENABLER0: [WriteOnly<u32>; 0x20],   // 0x180
//     GICD_ISPENDR0: [WriteOnly<u32>; 0x20],     // 0x200
//     GICD_ICPENDR0: [WriteOnly<u32>; 0x20],     // 0x280
//     GICD_ISACTIVER0: [WriteOnly<u32>; 0x20],   // 0x300
//     GICD_ICACTIVER0: [WriteOnly<u32>; 0x20],   // 0x380
//     GICD_IPRIORITYR0: [WriteOnly<u32>; 0x100], // 0x400
//     GICD_ITARGETSR0: [WriteOnly<u32>; 0x100],  // 0x800
//     GICD_ICFGR0: [WriteOnly<u32>; 0x40],       // 0xc00
//     __reserved_1: [WriteOnly<u32>; 0x80],      // 0xc40
//     GICD_SGIR: [WriteOnly<u32>; 0x40],         // 0xf00
//     GICC_CTLR: WriteOnly<u32>,                 // 0x1000
//     GICC_PMR: WriteOnly<u32>,                  // 0x1004
//     __reserved_2: u32,                         // 0x1008
//     GICC_IAR: WriteOnly<u32>,                  // 0x100c
//     GICC_EOIR: WriteOnly<u32>,                 // 0x1010
// }
//10d4
//1014
#[derive(Default)]
pub struct GICv2 {
    //irq_handlers: [Option<fn(&mut super::ExceptionContext)>; IRQ_LINES],
}

impl GICv2 {
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

fn addressof<T>(t: &T) -> u64 {
    t as *const T as u64
}

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

        crate::println!("{:x}, offset: 0x000", addressof(&self.GICD_CTLR));
        crate::println!("{:x}, offset: 0x080", addressof(&self.GICD_IGROUPR0));
        crate::println!("{:x}, offset: 0x100", addressof(&self.GICD_ISENABLER0));
        crate::println!("{:x}, offset: 0x180", addressof(&self.GICD_ICENABLER0));
        crate::println!("{:x}, offset: 0x200", addressof(&self.GICD_ISPENDR0));
        crate::println!("{:x}, offset: 0x280", addressof(&self.GICD_ICPENDR0));
        crate::println!("{:x}, offset: 0x300", addressof(&self.GICD_ISACTIVER0));
        crate::println!("{:x}, offset: 0x380", addressof(&self.GICD_ICACTIVER0));
        crate::println!("{:x}, offset: 0x400", addressof(&self.GICD_IPRIORITYR0));
        crate::println!("{:x}, offset: 0x800", addressof(&self.GICD_ITARGETSR0));
        crate::println!("{:x}, offset: 0xc00", addressof(&self.GICD_ICFGR0));
        crate::println!("{:x}, offset: 0xf00", addressof(&self.GICD_SGIR));
        crate::println!("{:x}, offset: 0x1000", addressof(&self.GICC_CTLR));
        crate::println!("{:x}, offset: 0x1004", addressof(&self.GICC_PMR));

        super::enable_irqs();

        Ok(())
    }
    fn enable_irq(&mut self, irq_number: usize) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        self.GICD_ISENABLER0[irq_number / 32].set(1 << (irq_number as u32 % 32));
        Ok(())
    }
    fn disable_irq(&mut self, irq_number: usize) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        self.GICD_ICENABLER0[irq_number / 32].set(1 << (irq_number as u32 % 32));
        Ok(())
    }
    fn connect_irq(
        &mut self,
        irq_number: usize,
        _handler: Option<fn(data: &mut ExceptionContext)>,
    ) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        //self.irq_handlers[irq_number as usize] = handler;
        self.enable_irq(irq_number)
    }
    fn disconnect_irq(&mut self, irq_number: usize) -> InteruptResult {
        if irq_number >= IRQ_LINES {
            return Err(InteruptError::IncorrectIrqNumber);
        }
        self.disable_irq(irq_number)?;
        //self.irq_handlers[irq_number] = None;
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
