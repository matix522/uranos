#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![feature(global_asm)]
#![feature(asm)]

//! Low-level crate for aarch64 support

/// Module containg varius inline asm functions for aarch64 architecture.
pub mod asm {
    #[inline(always)]
    ///Assembly nop (No operation) instruction
    pub fn nop() {
        unsafe {
            asm!("nop" : : : : "volatile");
        }
    }
    #[inline(always)]
    ///Assembly wfe (Wait for event) instruction
    pub fn wfe() {
        unsafe {
            asm!("wfe" : : : : "volatile");
        }
    }
    #[inline(always)]
    ///Assembly eret (Exception return) instruction
    pub fn eret() -> ! {
        unsafe {
            asm!("eret" : : : : "volatile");
        }
        loop {}
    }
    #[inline(always)]
    ///Set Stack Pointer of Kernel Mode
    pub fn set_el1_stack_pointer(sp: u64) {
        unsafe {
            asm!("msr sp_el1, $0" :  : "r"(sp) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set System Control Register for Kernel Mode
    pub fn set_el1_system_control_register(sctrl: u64) {
        unsafe {
            asm!("msr sctlr_el1, $0" :  :  "r"(sctrl) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Hypervisor Configuration Register
    pub fn set_el2_configuration_register(hcr: u64) {
        unsafe {
            asm!("msr hcr_el2, $0" :  : "r"(hcr) : : "volatile");
        }
    }

    #[inline(always)]
    ///Set Saved Program Status Register for Hypervisor
    pub fn set_el2_saved_program_status_register(spsr: u64) {
        unsafe {
            asm!("msr spsr_el2, $0" :  : "r"(spsr) : : "volatile");
        }
    }
    #[inline(always)]
    ///Set Saved Program Status Register for Hypervisor
    pub fn set_el2_exception_return_adrress(spsr: u64) {
        unsafe {
            asm!("msr elr_el2, $0" :  : "r"(spsr) : : "volatile");
        }
    }


}