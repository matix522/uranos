pub struct AddressSpaceId(usize);

pub enum KernelContext {
    AllocatorAddress,
    TaskAddress,
}
pub enum Address {
    Kernel(usize, KernelContext),
    User(usize, AddressSpaceId),
    Physical(usize),
}
