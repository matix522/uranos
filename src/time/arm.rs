use core::ops::Deref;
use register::mmio::*;
const ARM_CLOCK_BASE: usize = 0x4000_0040;

pub struct Registers {
    route_clock: WriteOnly<u32>,
}

pub struct ArmTimer;

impl Deref for ArmTimer {
    type Target = Registers;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(ARM_CLOCK_BASE as *const Registers) }
    }
}

impl super::Timer for ArmTimer {
    fn enable() {
        ArmTimer.route_clock.set(0x8);
        let val: u32 = 1;
        unsafe {
            llvm_asm!("msr cntv_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    fn disable() {
        ArmTimer.route_clock.set(0x0);
        let val: u32 = 0;
        unsafe {
            llvm_asm!("msr cntv_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    fn get_frequency() -> u32 {
        let frequency;
        unsafe {
            llvm_asm!("mrs $0, cntfrq_el0" : "=r"(frequency) : : : "volatile");
        }
        frequency
    }
    fn interupt_after(ticks: u32) -> Result<(), & 'static str> {
        unsafe {
            llvm_asm!("msr cntv_tval_el0, $0" : : "r"(ticks) : : "volatile");
        }
        Ok(())
    }
    fn get_time() -> u64 {
        let ticks;
        unsafe {
            llvm_asm!("mrs $0, cntvct_el0" : "=r" (ticks));
        }
        ticks
    }
}
