pub mod d_virtual;
pub mod physical;

pub trait Device {
    fn device_type(&self) -> &str;
    fn init(&self) -> Result<(), &str>;
    fn destroy(&self) -> Result<(), &str>;
}
