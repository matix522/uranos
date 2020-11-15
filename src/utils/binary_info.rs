use core::ops::Range;
extern "C" {
    pub fn _boot_cores() -> !;
    pub static __exception_vector_start: u64;
    pub static __binary_end: u64;
    pub static __read_only_start: usize;
    pub static __read_only_end: usize;
    pub static __read_write_start: usize;
    pub static __read_write_end: usize;
    pub static __task_local_start: usize;
    pub static __task_local_end: usize;
}

#[derive(Debug)]
pub struct BinaryInfo {
    pub binary: Range<usize>,
    pub read_only: Range<usize>,
    pub read_write: Range<usize>,
    pub task_local: Range<usize>,
    pub exception_vector: usize,
    pub allocator: Range<usize>,
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
                task_local: &__task_local_start as *const _ as usize
                    ..&__task_local_end as *const _ as usize,
                exception_vector: &__exception_vector_start as *const _ as usize,
                allocator: {
                    let alloc = &crate::memory::allocator::kernel_heap_range().start - 0x1000;
                    alloc as usize..alloc + 0x1000 as usize
                },
                heap: crate::memory::allocator::kernel_heap_range(),
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
            "\tTask Local Range: [{:#10x} - {:#10x}]",
            self.task_local.start, self.task_local.end
        )?;
        writeln!(
            f,
            "\tRead Write Range: [{:#10x} - {:#10x}]",
            self.read_write.start, self.read_write.end
        )?;
        writeln!(
            f,
            "\tException Vector: [{:#10x}            ]",
            self.exception_vector
        )?;
        writeln!(
            f,
            "\tMain Alocator:    [{:#10x} - {:#10x}]",
            self.allocator.start, self.allocator.end
        )?;
        writeln!(
            f,
            "\tMain Heap:        [{:#10x} - {:#10x}]",
            self.heap.start, self.heap.end
        )?;
        writeln!(
            f,
            "\tMMIO:             [{:#10x} - {:#10x}]",
            self.mmio.start, self.mmio.end
        )?;
        Ok(())
    }
}
