pub mod arm;
// pub mod bcmclock;

use crate::interupt::Error;

trait Timer {
    fn get_time() -> u64;
    fn interupt_after(ticks: u32) -> Result<(), Error>;
    fn enable();
    fn disable();
    fn get_frequency() -> u32;
}
