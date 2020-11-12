use super::async_syscall::*;
use crate::alloc::collections::BTreeMap;

pub struct AsyncReturnedValues {
    pub map: BTreeMap<usize, (AsyncSyscalls, usize)>,
}

impl AsyncReturnedValues {
    pub fn new() -> Self {
        AsyncReturnedValues {
            map: BTreeMap::<usize, (AsyncSyscalls, usize)>::new(),
        }
    }
}
