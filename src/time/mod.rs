pub mod arm;
// pub mod bcmclock;


trait Timer {
    fn get_time() -> u64;
    fn interupt_after(ticks: u32) -> Result<(),&'static str>;
    fn enable();
    fn disable();
    fn get_frequency() -> u32;
}
