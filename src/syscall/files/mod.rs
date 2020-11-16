pub mod close;
pub mod create;
pub mod delete;
pub mod file_descriptor_map;
pub mod open;
pub mod read;
pub mod seek;
pub mod write;

pub const PIPE_QUEUE_GRANULATION: usize = 64;

pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const PIPEIN: usize = 2;
pub const PIPEOUT: usize = 3;

use crate::interupts::ExceptionContext;
use crate::scheduler;
use crate::scheduler::task_context::TaskContext;
use crate::syscall::asynchronous;
use crate::syscall::asynchronous::files::{AsyncFileDescriptor, AsyncOpenedFile};
use crate::utils::circullar_buffer::CircullarBuffer;
use crate::vfs::{FileError, SeekType};

pub fn handle_set_pipe_read_on_pid(e: &mut ExceptionContext) {
    let pid = e.gpr[0] as usize;
    let current_task = scheduler::get_current_task_context();
    unsafe {
        (*current_task).pipe_from = Some(pid);
    }
}

pub fn resolve_fd(fd: usize) -> usize {
    let current_task: &mut TaskContext = unsafe { &mut *(scheduler::get_current_task_context()) };
    match current_task.mapped_fds.get(&fd) {
        Some(mapped_fd) => *mapped_fd,
        None => fd,
    }
}

pub struct File {
    fd: usize,
}

impl File {
    pub fn open(filename: &str, with_write: bool) -> Result<Self, FileError> {
        let fd = open::open(filename, with_write)?;
        Ok(File { fd })
    }
    pub fn async_open(
        filename: &'static str,
        with_write: bool,
        id: usize,
        submission_buffer: &mut CircullarBuffer,
    ) -> AsyncOpenedFile {
        asynchronous::files::open::open(filename, with_write, id, submission_buffer)
    }
    pub fn read(&self, length: usize, buffer: &mut [u8]) -> Result<usize, FileError> {
        read::read(self.fd, length, buffer as *mut [u8] as *mut u8)
    }
    pub fn async_read(
        &self,
        length: usize,
        buffer: &mut [u8],
        id: usize,
        submission_buffer: &mut CircullarBuffer,
    ) -> AsyncOpenedFile {
        asynchronous::files::read::read(
            &AsyncFileDescriptor::FileDescriptor(self.fd),
            length,
            buffer as *mut [u8] as *mut u8,
            id,
            submission_buffer,
        )
    }
    pub fn write(&self, bytes: &[u8]) -> Result<(), FileError> {
        write::write(self.fd, bytes)
    }
    pub fn async_write(
        &self,
        message: &'static [u8],
        id: usize,
        submission_buffer: &mut CircullarBuffer,
    ) -> AsyncOpenedFile {
        asynchronous::files::write::write(
            &AsyncFileDescriptor::FileDescriptor(self.fd),
            message,
            id,
            submission_buffer,
        )
    }
    pub fn seek(&self, value: isize, seek_type: SeekType) -> Result<usize, FileError> {
        seek::seek(self.fd, value, seek_type)
    }
    pub fn async_seek(
        &self,
        value: isize,
        seek_type: SeekType,
        id: usize,
        submission_buffer: &mut CircullarBuffer,
    ) -> AsyncOpenedFile {
        asynchronous::files::seek::seek(
            &AsyncFileDescriptor::FileDescriptor(self.fd),
            value,
            seek_type,
            id,
            submission_buffer,
        )
    }
    pub fn close(&self) -> Result<(), FileError> {
        close::close(self.fd)
    }
    pub fn async_close(&self, id: usize, submission_buffer: &mut CircullarBuffer) {
        asynchronous::files::close::close(
            &AsyncFileDescriptor::FileDescriptor(self.fd),
            id,
            submission_buffer,
        )
    }
    pub fn get_fd(&self) -> usize {
        self.fd
    }

    pub fn get_stdin() -> Self {
        File { fd: STDIN }
    }

    pub fn get_stdout() -> Self {
        File { fd: STDOUT }
    }

    pub fn get_pipein() -> Self {
        File { fd: PIPEIN }
    }

    pub fn get_pipeout() -> Self {
        File { fd: PIPEOUT }
    }
}
