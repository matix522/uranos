use crate::vfs::*;
use alloc::collections::BTreeMap;

pub struct FileDescriptiorMap {
    map: BTreeMap<usize, OpenedFile>,
    next_fd: usize,
}

impl Default for FileDescriptiorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl FileDescriptiorMap {
    pub fn new() -> Self {
        FileDescriptiorMap {
            map: BTreeMap::new(),
            next_fd: 1,
        }
    }

    pub fn add_file(&mut self, file: OpenedFile) -> usize {
        let ret = self.next_fd;
        self.map.insert(ret, file);
        self.next_fd = match self.map.keys().max() {
            Some(val) => val + 1,
            None => 1,
        };
        ret
    }

    pub fn get_file_mut(&mut self, fd: usize) -> Option<&mut OpenedFile> {
        self.map.get_mut(&fd)
    }

    pub fn get_file(&mut self, fd: usize) -> Option<&OpenedFile> {
        self.map.get(&fd)
    }

    pub fn exists(&mut self, fd: usize) -> bool {
        self.map.get(&fd).is_some()
    }

    pub fn delete_file(&mut self, fd: usize) -> Option<OpenedFile> {
        self.map.remove(&fd)
    }
}
