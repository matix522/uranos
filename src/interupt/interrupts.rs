use super::{MMIO_BASE, PERIPHERALS_BASE};
use register::{mmio::ReadWrite, mmio::WriteOnly, register_bitfields};

const INT_TYPES: &'static [&'static str] = &[
    "SYNC_INVALID_EL1t",
	"IRQ_INVALID_EL1t",		
	"FIQ_INVALID_EL1t",		
	"ERROR_INVALID_EL1T",		

	"SYNC_INVALID_EL1h",		
	"IRQ_INVALID_EL1h",		
	"FIQ_INVALID_EL1h",		
	"ERROR_INVALID_EL1h",		

	"SYNC_INVALID_EL0_64",		
	"IRQ_INVALID_EL0_64",		
	"FIQ_INVALID_EL0_64",		
	"ERROR_INVALID_EL0_64",	

	"SYNC_INVALID_EL0_32",		
	"IRQ_INVALID_EL0_32",		
	"FIQ_INVALID_EL0_32",		
	"ERROR_INVALID_EL0_32"	
];

register_bitfields!{
    u32,
    /// Register for enabling/disabling peripherals interrupts
    ENABLE_IRQ_1[
        ///System timer IRQ toggle
        SYSTEM_TIMER_IRQ_1 OFFSET(1) NUMBITS(1)[
            Enable = 0b1,
            Disable = 0b0
        ]
    ]
}


// .global __irq_vector_init
// __irq_vector_init:
//     adr x0, __exception_vectors_start
//     msr vbar_el1, x0
//     ret

pub fn init_IRQ_vector() -> !{
    unsafe{
        asm!("adr x0, __exception_vectors_start" :::: "volatile");
        asm!("msr vbar_el1, x0" :::: "volatile");
    }
}


pub const ENABLE_IRQ_1: *const ReadWrite<u32, ENABLE_IRQ_1::Register> =
    (PERIPHERALS_BASE + 0x0000_B210) as *const ReadWrite<u32, ENABLE_IRQ_1::Register>;

pub fn show_invalid_entry_message(type: u32, esr: u64, address: u64) -> !{
    println!("Invalid interrupt entry: {}, ESR: {}, address: {x}", INT_TYPES[type], esr, address);
    gpio::blink();
}
pub fn handle_invalid_entry_message(type: u32, esr: u64, address: u64) -> !{
    show_invalid_entry_message(type, esr, address);
    gpio::blink();
}

pub fn enable_interrupt_controller() -> !{
    (*ENABLE_IRQ_1).modify(ENABLE_IRQ_1::SYSTEM_TIMER_IRQ_1::Enable);
}