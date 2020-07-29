use core::ops::Deref;
use register::mmio::*;
const ARM_CLOCK_BASE: usize = 0x4000_0040;

use core::time::Duration;

use crate::drivers::traits::time;

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

impl time::Timer for ArmTimer {
    fn get_time_raw(&self) -> u64 {
        let ticks;
        unsafe {
            llvm_asm!("mrs $0, cntvct_el0" : "=r" (ticks));
        }
        ticks
    }
    fn get_time(&self) -> Duration {
        let ticks = self.get_time_raw() as u128;
        let freq = self.get_frequency() as u128;
        let micros = (ticks * 1_000_000) / freq;
        Duration::from_micros(micros as u64)
    }
    fn get_frequency(&self) -> u32 {
        let frequency;
        unsafe {
            llvm_asm!("mrs $0, cntfrq_el0" : "=r"(frequency) : : : "volatile");
        }
        frequency
    }
    fn interupt_after_raw(&self, ticks: u32) {
        unsafe {
            llvm_asm!("msr cntv_tval_el0, $0" : : "r"(ticks) : : "volatile");
        }
    }
    fn interupt_after(&self, time: Duration) {
        let freq = self.get_frequency() as u128;
        let ticks = ((time.as_micros() * freq) / 1_000_000) as u32;
        self.interupt_after_raw(ticks);
    }
    fn enable(&self) {
        ArmTimer.route_clock.set(0x8);
        let val: u32 = 1;
        unsafe {
            llvm_asm!("msr cntv_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    fn disable(&self) {
        ArmTimer.route_clock.set(0x0);
        let val: u32 = 0;
        unsafe {
            llvm_asm!("msr cntv_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    fn wait_raw(&self, ticks: u64) {
        let target = self.get_time_raw() + ticks;
        while self.get_time_raw() < target {}
    }
    fn wait(&self, time: Duration) {
        let freq = self.get_frequency() as u128;
        self.wait_raw(((time.as_micros() * freq) / 1_000_000) as u64);
    }
}
