use super::*;
use async_print::*;
use async_syscall::*;
use files::*;

pub fn handle_async_syscalls() {
    let current_task = unsafe { &mut *crate::scheduler::get_current_task_context() };
    while !current_task.submission_buffer.is_empty() {
        let syscall_ret_opt = crate::syscall::asynchronous::async_syscall::read_async_syscall(
            &mut current_task.submission_buffer,
        );
        if let Some(syscall_ret) = syscall_ret_opt {
            //ommit the syscall type value that is at the beginning of the data from buffer
            let data = syscall_ret.get_syscall_data();
            let ptr = data as *const _ as *const u8;
            let length = syscall_ret.get_data_size();
            let returned_value = match syscall_ret.syscall_type {
                AsyncSyscalls::Print => handle_async_print(ptr, length),
                AsyncSyscalls::OpenFile => {
                    let ret = open::handle_async_open(ptr, length);
                    current_task
                        .async_returns_map
                        .map
                        .insert(syscall_ret.id, (syscall_ret.syscall_type, ret));
                    ret
                }
                AsyncSyscalls::ReadFile => {
                    read::handle_async_read(ptr, length, &mut current_task.async_returns_map)
                }
                AsyncSyscalls::SeekFile => {
                    seek::handle_async_seek(ptr, length, &mut current_task.async_returns_map)
                }
                AsyncSyscalls::WriteFile => {
                    write::handle_async_write(ptr, length, &mut current_task.async_returns_map)
                }
                AsyncSyscalls::CloseFile => {
                    current_task.async_returns_map.map.remove(&syscall_ret.id);
                    close::handle_async_close(ptr, length, &mut current_task.async_returns_map)
                }
            };

            let buffer_frame = current_task
                .completion_buffer
                .reserve(core::mem::size_of::<AsyncSyscallReturnedValue>())
                .expect("Error during sending async syscall response");
            let return_structure: &mut AsyncSyscallReturnedValue =
                unsafe { crate::utils::struct_to_slice::u8_slice_to_any_mut(buffer_frame.memory) };
            return_structure.id = syscall_ret.id;
            return_structure.value = returned_value;
        }
    }
    current_task.update_zombie();
}
