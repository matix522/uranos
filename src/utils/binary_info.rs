extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vector_start: u64;
    pub static __binary_end: u64;
    pub static __read_only_start: usize;
    pub static __read_only_end: usize;
}

#[derive(Debug)]
pub struct BinaryInfo {
    pub binary_start: usize,
    pub binary_end: usize,
    pub read_only_start: usize,
    pub read_only_end: usize,
    pub exception_vector: usize,
    pub heap_start: usize,
    pub heap_end: usize,
}
impl BinaryInfo {
    pub fn get() -> BinaryInfo {
        unsafe {
            BinaryInfo {
                binary_start: _boot_cores as *const () as usize,
                binary_end: &__binary_end as *const _ as usize,
                read_only_start: &__read_only_start as *const _ as usize,
                read_only_end: &__read_only_end as *const _ as usize,
                exception_vector: &__exception_vector_start as *const _ as usize,
                heap_start: crate::memory::allocator::heap_start(),
                heap_end: crate::memory::allocator::heap_end(),
            }
        }
    }
}
use core::fmt;
impl fmt::Display for BinaryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "Binary definition:")?;
        writeln!(
            f,
            "\tRange:            [{:#10x}  {:#10x}]",
            self.binary_start, self.binary_end
        )?;
        writeln!(
            f,
            "\tRead Only Range:  [{:#10x}  {:#10x}]",
            self.read_only_start, self.read_only_end
        )?;
        writeln!(
            f,
            "\tException Vector: [{:#10x}            ]",
            self.exception_vector
        )?;
        writeln!(
            f,
            "\tMain Heap:        [{:#10x}  {:#10x}]",
            self.heap_start, self.heap_end
        )?;
        Ok(())
    }
}
