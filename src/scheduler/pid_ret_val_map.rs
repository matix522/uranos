use super::async_syscall::*;
use crate::alloc::collections::BTreeMap;


pub struct PIDRetValMap{
    map: BTreeMap<usize, Option<u64>>,    
}

impl PIDRetValMap{
    pub fn new() -> Self{
        PIDRetValMap{
            map: BTreeMap::<usize, Option<u64>>::new();
        }
    }
}