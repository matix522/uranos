pub mod global;

pub fn delay(ticks: usize) {
    for _ in 0..ticks {
        crate::aarch64::asm::nop();
    }
}
