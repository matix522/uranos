pub mod syscall;
pub use num_traits::FromPrimitive;

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum Syscalls {
    Print
}