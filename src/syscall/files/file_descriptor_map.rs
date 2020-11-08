use crate::device_driver;
use crate::vfs::*;
use alloc::collections::BTreeMap;

pub struct FileDescriptiorMap {
    map: BTreeMap<isize, OpenedFile>,
    next_fd: isize,
}

impl<'a> FileDescriptiorMap {
    pub fn new() -> Self {
        FileDescriptiorMap {
            map: BTreeMap::new(),
            next_fd: 1,
        }
    }

    pub fn add_file(&mut self, file: OpenedFile) -> isize {
        let ret = self.next_fd;
        self.map.insert(ret, file);
        self.next_fd = match self.map.keys().max() {
            Some(val) => val + 1,
            None => 1,
        };
        ret
    }

    pub fn get_file(&mut self, fd: isize) -> Option<&mut OpenedFile> {
        self.map.get_mut(&fd)
    }

    pub fn delete_file(&mut self, fd: isize) {
        self.map.remove(&fd);
    }
}
