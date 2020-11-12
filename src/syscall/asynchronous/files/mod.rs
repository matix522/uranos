pub mod close;
pub mod open;
pub mod read;
pub mod seek;
pub mod write;
use crate::utils::circullar_buffer::CircullarBuffer;
use crate::vfs;

pub const ONLY_MSB_OF_USIZE: usize = 1 << (core::mem::size_of::<usize>() * 8 - 1);

pub enum AsyncFileDescriptor {
    FileDescriptor(usize),
    AsyncSyscallReturnValue(usize),
}

impl AsyncFileDescriptor {
    pub fn from_usize(val: usize) -> Self {
        if val & ONLY_MSB_OF_USIZE > 0 {
            AsyncFileDescriptor::AsyncSyscallReturnValue(val & !ONLY_MSB_OF_USIZE)
        } else {
            AsyncFileDescriptor::FileDescriptor(val)
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            AsyncFileDescriptor::FileDescriptor(val) => val & !ONLY_MSB_OF_USIZE,
            AsyncFileDescriptor::AsyncSyscallReturnValue(val) => val | ONLY_MSB_OF_USIZE,
        }
    }
}

pub struct AsyncOpenedFile {
    pub afd: AsyncFileDescriptor,
}

impl AsyncOpenedFile {
    pub fn then_read(
        &self,
        length: usize,
        buffer: *mut u8,
        id: usize,
        submission_buffer: &mut CircullarBuffer,
    ) -> &AsyncOpenedFile {
        read::read(&self.afd, length, buffer, id, submission_buffer);
        self
    }
    pub fn then_seek(
        &self,
        value: isize,
        seek_type: vfs::SeekType,
        id: usize,
        submission_buffer: &mut CircullarBuffer,
    ) -> &AsyncOpenedFile {
        seek::seek(&self.afd, value, seek_type, id, submission_buffer);
        self
    }
    pub fn then_write(
        &self,
        message: &'static [u8],
        id: usize,
        submission_buffer: &mut CircullarBuffer,
    ) -> &AsyncOpenedFile {
        write::write(&self.afd, message, id, submission_buffer);
        self
    }
    pub fn then_close(&self, id: usize, submission_buffer: &mut CircullarBuffer) {
        close::close(&self.afd, id, submission_buffer);
    }
}
