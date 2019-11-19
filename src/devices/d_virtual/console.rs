pub enum WriteError {}

pub enum ReadError {}

pub trait Console {
    fn read(&mut self, buffer: &[u8]) -> Result<usize, ReadError>;
    fn write(&mut self, buffer: &[u8]) -> Result<usize, WriteError>;
}
