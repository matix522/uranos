pub mod arm;
// pub mod bcmclock;

pub trait Timer {
    fn get_time() -> u64;
    fn interupt_after(ticks: u32);
    fn enable();
    fn disable();
    fn get_frequency() -> u32;
}
