use core::ops::Range;
extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vector_start: u64;
    pub static __binary_end: u64;
    pub static __read_only_start: usize;
    pub static __read_only_end: usize;
    pub static __read_write_start: usize;
    pub static __read_write_end: usize;
}

#[derive(Debug)]
pub struct BinaryInfo {
    pub binary: Range<usize>,
    pub read_only: Range<usize>,
    pub read_write: Range<usize>,
    pub exception_vector: usize,
    pub heap: Range<usize>,
    pub mmio: Range<usize>,
}
impl BinaryInfo {
    pub fn get() -> BinaryInfo {
        unsafe {
            BinaryInfo {
                binary: _boot_cores as *const () as usize..&__binary_end as *const _ as usize,
                read_only: &__read_only_start as *const _ as usize
                    ..&__read_only_end as *const _ as usize,
                read_write: &__read_write_start as *const _ as usize
                    ..&__read_write_end as *const _ as usize,
                exception_vector: &__exception_vector_start as *const _ as usize,
                heap: crate::memory::allocator::heap_base()..crate::memory::allocator::heap_end(),
                mmio: crate::memory::physical::mmio::BASE..crate::memory::physical::mmio::END,
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
            "\tRange:            [{:#10x} - {:#10x}]",
            self.binary.start, self.binary.end
        )?;
        writeln!(
            f,
            "\tRead Only Range:  [{:#10x} - {:#10x}]",
            self.read_only.start, self.read_only.end
        )?;
        writeln!(
            f,
            "\tRead Write Range:  [{:#10x} - {:#10x}]",
            self.read_write.start, self.read_write.end
        )?;
        writeln!(
            f,
            "\tException Vector: [{:#10x}            ]",
            self.exception_vector
        )?;
        writeln!(
            f,
            "\tMain Heap:        [{:#10x} - {:#10x}]",
            self.heap.start, self.heap.end
        )?;
        Ok(())
    }
}
