#[cfg(not(feature = "raspi4"))]
use core::ops::Deref;
#[cfg(not(feature = "raspi4"))]
use register::mmio::*;
#[cfg(not(feature = "raspi4"))]
pub struct RegisterBlocArm {
    
    route_clock: WriteOnly<u32>,
}
#[cfg(not(feature = "raspi4"))]
const ARM_CLOCK_BASE: usize = 0x40000040;

pub struct ArmQemuTimer;
#[cfg(not(feature = "raspi4"))]
impl Deref for ArmQemuTimer {
    type Target = RegisterBlocArm;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(ARM_CLOCK_BASE as *const RegisterBlocArm) }
    }
}

#[allow(unused)]
fn dummy(data: &mut super::ExceptionContext) {}
#[cfg(feature = "raspi4")]
impl ArmQemuTimer {
    pub fn enable() {
        use super::gicv2::GICv2;
        use super::InteruptController;
        //pub fn enable(interupt_controller : &impl InteruptController) {
        let mut interupt_controller = GICv2 {};
        interupt_controller.connect_irq(30, Some(dummy)).unwrap();
        interupt_controller.connect_irq(29, Some(dummy)).unwrap();
        interupt_controller.connect_irq(27, Some(dummy)).unwrap();
        interupt_controller.connect_irq(26, Some(dummy)).unwrap();

        let val: u32 = 1;
        unsafe {
            asm!("msr cntp_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    pub fn disable() {
        use super::gicv2::GICv2;
        use super::InteruptController;
        //pub fn disable(interupt_controller : &impl InteruptController) {
        let mut interupt_controller = GICv2 {};
        interupt_controller.disconnect_irq(30).unwrap();
        interupt_controller.disconnect_irq(29).unwrap();
        interupt_controller.disconnect_irq(27).unwrap();
        interupt_controller.disconnect_irq(26).unwrap();
        let val: u32 = 0;
        unsafe {
            asm!("msr cntp_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    pub fn get_frequency() -> u32 {
        let frequency;
        unsafe {
            asm!("mrs $0, cntfrq_el0" : "=r"(frequency) : : : "volatile");
        }
        frequency
    }
    pub fn interupt_after(ticks: u32) {
        let x = ticks as u64 + Self::get_time();
        unsafe {
            asm!("msr cntp_cval_el0, $0" : : "r"(x) : : "volatile");
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
            asm!("mrs $0, cntpct_el0" : "=r" (ticks) : : : "volatile");
        }
        ticks
    }
}
#[cfg(not(feature = "raspi4"))]
impl ArmQemuTimer {
    pub fn enable() {
        ArmQemuTimer.route_clock.set(0x8);
        let val: u32 = 1;
        unsafe {
            asm!("msr cntv_ctl_el0, $0" : : "r"(val) : : "volatile");
        }
    }
    pub fn disable() {
        ArmQemuTimer.route_clock.set(0x0);
        let val: u32 = 0;
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
    pub fn interupt_after(ticks: u32) {
        let x = ticks as u64 + Self::get_time();
        unsafe {
            asm!("msr cntv_cval_el0, $0" : : "r"(x) : : "volatile");
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
