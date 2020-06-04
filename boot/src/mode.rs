use aarch64::*;

#[derive(Debug)]
/// Level on which CPU is working
pub enum ExceptionLevel {
    /// EL0
    User,
    /// EL1
    Kernel,
    /// EL2
    Hypervisor,
    /// EL3
    Firmware,
}


const SCTLR_RESERVED: u64 = 3 << 28 | 3 << 22 | 1 << 20 | 1 << 11;
const SCTLR_EE_LITTLE_ENDIAN: u64 = 0;

const SCTLR_I_CACHE_DISABLED: u64 = 0;
const SCTLR_D_CACHE_DISABLED: u64 = 0;
const SCTLR_MMU_DISABLED: u64 = 0;


const SCTLR_VALUE_MMU_DISABLED: u64 = SCTLR_RESERVED
    | SCTLR_EE_LITTLE_ENDIAN
    | SCTLR_I_CACHE_DISABLED
    | SCTLR_D_CACHE_DISABLED
    | SCTLR_MMU_DISABLED;

const HCR_RW: u64 = 1 << 31;
const HCR_VALUE: u64 = HCR_RW;

const SCR_RESERVED: u64 = 3 << 4;
const SCR_RW: u64 = 1 << 10;
const SCR_NS: u64 = 1;
const SCR_VALUE: u64 = SCR_RESERVED | SCR_RW | SCR_NS;


const SPSR_MASK_ALL: u64 = 7 << 6;
const SPSR_EL1H: u64 = 5;
const SPSR_VALUE: u64 = SPSR_MASK_ALL | SPSR_EL1H;


impl ExceptionLevel {
    /// Retrive current level from register
    pub fn get_current() -> ExceptionLevel {
        let mut level : u64;
        unsafe { 
            asm!("mrs $0, CurrentEL" : "=r"(level) : : : "volatile"); 
        }
        match level >> 2 {
            0 => ExceptionLevel::User,
            1 => ExceptionLevel::Kernel,
            2 => ExceptionLevel::Hypervisor,
            3 => ExceptionLevel::Firmware,
            _ => halt()
        }
    }

    /// Assuming that current execution level is higer than EL1 drops to it.
    /// Takes a pointer to function that will be executed in EL1
    /// # Safety 
    /// Function must only be called during system startup in either EL2 or EL3 
    pub unsafe fn drop_to_el1(el1_entry : unsafe fn () -> !) -> ! {
        const STACK_START: u64 = 0x80_000;

        asm::initialize_timers_el1();

        match ExceptionLevel::get_current() {
            Self::User => halt(),
            Self::Kernel => halt(),
            Self::Hypervisor => {
                
                asm::set_el1_stack_pointer(STACK_START);
                asm::set_el1_system_control_register(SCTLR_VALUE_MMU_DISABLED);
                asm::set_el2_configuration_register(HCR_VALUE);
                asm::set_el2_saved_program_status_register(SPSR_VALUE);
                asm::set_el2_exception_return_adrress(el1_entry as *const () as u64);
            },
            Self::Firmware => {
                asm::set_el1_stack_pointer(STACK_START);
                asm::set_el1_system_control_register(SCTLR_VALUE_MMU_DISABLED);
                asm::set_el2_configuration_register(HCR_VALUE);
                asm::set_el3_configuration_register_safe(SCR_VALUE);
                asm::set_el3_saved_program_status_register(SPSR_VALUE);
                asm::set_el3_exception_return_adrress(el1_entry as *const () as u64);
            }
        }
        asm::eret();

    }
}




