use crate::utils::circullar_buffer::*;
use num_traits::FromPrimitive;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum AsyncSyscalls {
    Print,
    OpenFile,
    ReadFile,
    SeekFile,
    WriteFile,
}

pub struct AsyncSyscall<'a> {
    pub id: usize,
    pub data_size: usize,
    pub syscall_type: AsyncSyscalls,
    pub data: &'a [u8],
}

pub struct AsyncSyscallRequest<'a> {
    pub id: usize,
    pub syscall_type: AsyncSyscalls,
    pub data: ReturnedValue<'a>,
}

pub struct AsyncSyscallReturnedValue {
    pub id: usize,
    pub value: usize,
}

impl<'a> AsyncSyscallRequest<'a> {
    pub fn get_syscall_data(&self) -> &'a [u8] {
        &self.data.memory[2 * core::mem::size_of::<usize>()..]
    }
    pub fn get_data_size(&self) -> usize {
        self.data.get_size() - 2 * core::mem::size_of::<usize>()
    }
}

pub fn send_async_syscall(buffer: &mut CircullarBuffer, syscall: AsyncSyscall) {
    let usize_size = core::mem::size_of::<usize>();
    let mut buffer_frame = buffer
        .reserve(2 * usize_size + syscall.data_size)
        .expect("Error during sending async syscall");
    unsafe {
        let mut pointer = &mut *buffer_frame as *mut _ as *mut usize;
        *pointer = syscall.syscall_type as usize;
        pointer = pointer.add(1);
        *pointer = syscall.id;

        core::ptr::copy_nonoverlapping(
            syscall.data as *const _ as *const u8,
            (&mut (*buffer_frame) as *mut _ as *mut u8).add(2 * usize_size),
            syscall.data_size,
        );
    }
}

pub fn read_async_syscall(buffer: &mut CircullarBuffer) -> Option<AsyncSyscallRequest> {
    if buffer.is_empty() {
        return None;
    }
    let buffer_entry: ReturnedValue = buffer
        .get_value()
        .expect("Error during reading async syscall");
    unsafe {
        let pointer = buffer_entry.get_ref() as *const _ as *const usize;
        let syscall_type_usize = *pointer;
        let syscall_type = AsyncSyscalls::from_usize(syscall_type_usize).unwrap();
        let syscall_id = *(pointer.add(1));

        Some(AsyncSyscallRequest {
            syscall_type,
            data: buffer_entry,
            id: syscall_id,
        })
    }
}

pub fn get_syscall_returned_value(
    completion_buffer: &mut CircullarBuffer,
) -> Option<&AsyncSyscallReturnedValue> {
    match completion_buffer.get_value() {
        Err(_) => None,
        Ok(returned_value) => Some(unsafe {
            crate::utils::struct_to_slice::u8_slice_to_any::<AsyncSyscallReturnedValue>(
                returned_value.memory,
            )
        }),
    }
}
