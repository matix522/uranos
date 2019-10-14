use register::{mmio::*, register_bitfields};
use core::ops::Deref;

pub struct RegisterBlocArm {
    route_clock: WriteOnly<u32>,
}
const ARM_CLOCK_BASE : usize = 0x40000040; 

pub struct ArmQemuTimer;

impl Deref for ArmQemuTimer {

    type Target = RegisterBlocArm;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(ARM_CLOCK_BASE as *const RegisterBlocArm) }
    }
}
impl ArmQemuTimer {
    pub fn enable(){
        ArmQemuTimer.route_clock.set(0x8);
        let val : u32 = 1;
        unsafe {
            asm!("msr cntv_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    pub fn disable(){
        ArmQemuTimer.route_clock.set(0x0);
        let val : u32 = 0;
        unsafe {
            asm!("msr cntv_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    pub fn get_frequency() -> u32 {
        let frequency;
        unsafe {
            asm!("mrs $0, cntfrq_el0" : "=r"(frequency) : : : "volatile");
        }
        frequency
    }
    pub fn interupt_after(ticks : u32) {
        unsafe {
        	asm!("msr cntv_tval_el0, $0" : : "r"(ticks) : : "volatile");
        }
    }
    pub fn ticks_to_interupt() -> u32 {
        let ticks;
        unsafe {
            asm!("mrs $0, cntfrq_el0" : "=r"(ticks) : : : "volatile");
        }
        ticks
    }
    pub fn get_time() -> u64 {
        let ticks;
        unsafe {
	        asm!("mrs $0, cntvct_el0" : "=r" (ticks));
        }
        ticks
    }
}