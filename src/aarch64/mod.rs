//! Low-level crate for aarch64 support

/// Module containg varius inline asm functions for aarch64 architecture.
pub mod asm {
    #[inline(always)]
    ///Assembly nop (No operation) instruction
    pub fn nop() {
        unsafe {
            llvm_asm!("nop" : : : : "volatile");
        }
    }
    #[inline(always)]
    ///Assembly wfe (Wait for event) instruction
    pub fn wfe() {
        unsafe {
            llvm_asm!("wfe" : : : : "volatile");
        }
    }
    #[inline(always)]
    ///Assembly eret (Exception return) instruction
    pub fn eret() -> ! {
        unsafe {
            llvm_asm!("eret" : : : : "volatile");
        }
        loop {
            wfe();
        }
    }
    #[inline(always)]
    ///Set Stack Pointer of Kernel Mode
    pub fn copy_el1_to_el0_stack_pointer() {
        unsafe {
            llvm_asm!("mov x0, sp
                  msr sp_el0, x0" : : : "x0": "volatile");
        }
    }
    #[inline(always)]
    ///Set Saved Program Status Register
    pub fn set_el0_saved_program_status_register(spsr: u64) {
        unsafe {
            llvm_asm!("msr spsr_el0, $0" : : "r"(spsr) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Stack Pointer of Kernel Mode
    pub fn set_el1_stack_pointer(sp: u64) {
        unsafe {
            llvm_asm!("msr sp_el1, $0" :  : "r"(sp) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set System Control Register for Kernel Mode
    pub fn set_el1_system_control_register(sctrl: u64) {
        unsafe {
            llvm_asm!("msr sctlr_el1, $0" :  :  "r"(sctrl) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Hypervisor Configuration Register
    pub fn set_el2_configuration_register(hcr: u64) {
        unsafe {
            llvm_asm!("msr hcr_el2, $0" :  : "r"(hcr) : : "volatile");
        }
    }

    #[inline(always)]
    ///Set Saved Program Status Register for Hypervisor
    pub fn set_el2_saved_program_status_register(spsr: u64) {
        unsafe {
            llvm_asm!("msr spsr_el2, $0" :  : "r"(spsr) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Saved Program Status Register for Hypervisor
    pub fn set_el2_exception_return_adrress(spsr: u64) {
        unsafe {
            llvm_asm!("msr elr_el2, $0" :  : "r"(spsr) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Configuration Register for Firmawre
    pub fn set_el3_configuration_register_safe(scr: u64) {
        unsafe {
            llvm_asm!("msr scr_el3, $0" :  : "r"(scr) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Saved Program Status Register for Firmawre
    pub fn set_el3_saved_program_status_register(spsr: u64) {
        unsafe {
            llvm_asm!("msr spsr_el3, $0" :  : "r"(spsr) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Saved Program Status Register for Firmawre
    pub fn set_el3_exception_return_adrress(spsr: u64) {
        unsafe {
            llvm_asm!("msr elr_el3, $0" :  : "r"(spsr) : : "volatile");
        }
    }
    /// enable usage of physical timer in el1
    #[inline(always)]
    pub fn initialize_timers_el1() {
        let _value: u64;
        unsafe {
            llvm_asm!("
            mrs	$0, cnthctl_el2
            orr	$0, $0, #0x3
            msr	cnthctl_el2, $0
            msr	cntvoff_el2, xzr"
            : "=r"(_value): : : "volatile");
        }
    }
}
#[inline(always)]
///Enters into unending loop of wfe instructions
pub fn halt() -> ! {
    loop {
        asm::wfe();
    }
}
