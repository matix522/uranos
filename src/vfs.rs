use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;

#[derive(Debug)]
pub enum FileError {
    FileNameAlreadyExists,
    FileDoesNotExist,
    AttemptToCloseClosedFile,
    PositionOutOfBoundsOfFile,
    ModifyingWithoutWritePermission,
}

pub struct File {
    pub data: String,
    pub is_opened_for_read: u16,
    pub is_opened_for_write: bool,
}

impl File {
    pub fn empty() -> Self {
        File {
            data: String::new(),
            is_opened_for_read: 0,
            is_opened_for_write: false,
        }
    }

    pub fn close(&mut self) -> Result<(), FileError> {
        if self.is_opened_for_write {
            self.is_opened_for_write = false;
        } else {
            if self.is_opened_for_read == 0 {
                return Err(FileError::AttemptToCloseClosedFile);
            }
            self.is_opened_for_read -= 1;
        }
        Ok(())
    }
}

pub struct OpenedFile<'a> {
    file: &'a mut File,
    read_pointer: usize,
}

impl<'a> OpenedFile<'a> {
    pub fn close(&mut self) -> Result<(), FileError> {
        self.file.close()
    }
    pub fn read(&mut self, length: usize) -> &str {
        let file_len = self.file.data.len();
        let end_of_read = if file_len < self.read_pointer + length {
            file_len
        } else {
            self.read_pointer + length
        };
        let result = &self.file.data[self.read_pointer..end_of_read];
        self.read_pointer = end_of_read;
        result
    }
    pub fn seek(&mut self, position: usize) -> Result<(), FileError> {
        if position >= self.len() {
            Err(FileError::PositionOutOfBoundsOfFile)
        } else {
            self.read_pointer = position;
            Ok(())
        }
    }

    pub fn append(&mut self, message: &str) -> Result<(), FileError> {
        if self.file.is_opened_for_write {
            self.file.data.push_str(message);
            Ok(())
        } else {
            Err(FileError::ModifyingWithoutWritePermission)
        }
    }

    pub fn len(&self) -> usize {
        self.file.data.len()
    }
}

impl<'a> Drop for OpenedFile<'a> {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct VFS {
    file_map: BTreeMap<String, File>,
}

impl VFS {
    pub fn new() -> Self {
        VFS {
            file_map: BTreeMap::new(),
        }
    }
    pub fn example_vfs() -> Self {
        let mut vfs = VFS::new();
        vfs.file_map.insert("file1".to_string(), File{
            data: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam volutpat posuere massa, quis feugiat diam consectetur eget. Quisque vitae feugiat odio. Pellentesque sed sem eu turpis aliquet lacinia. Nam facilisis finibus mi vitae dignissim. Praesent id nunc leo. Nulla non dapibus justo, quis sagittis est. Maecenas et lorem a nulla imperdiet facilisis ac sit amet nulla. Nulla facilisi. Fusce orci nibh, dapibus at rhoncus non, faucibus eget ipsum. Suspendisse potenti. Nunc tempor felis elit, rhoncus porta ante porttitor id. Ut viverra tincidunt feugiat. Curabitur enim elit, fringilla ac metus eget, vestibulum malesuada enim. Proin ac augue dignissim, egestas lacus eu, dictum eros. Suspendisse rutrum venenatis risus eleifend consectetur.".to_string(),
            is_opened_for_read: 0,
            is_opened_for_write: false});
        vfs.file_map.insert("file2".to_string(), File{
            data: "Bee Movie Script - Dialogue Transcript According to all known laws of aviation, there is no way a bee should be able to fly. Its wings are too small to get its fat little body off the ground. The bee, of course, flies anyway because bees don't care what humans think is impossible. Yellow, black. Yellow, black. Yellow, black. Yellow, black. Ooh, black and yellow! Let's shake it up a little. Barry! Breakfast is ready! Ooming!".to_string(),
            is_opened_for_read: 0,
            is_opened_for_write: false});
        vfs
    }

    // pub fn list_files(&self) -> Vec<String>{
    //     self.file_map.keys().copied().collect()
    // }

    pub fn add_file(&mut self, filename: &str) -> Result<(), FileError> {
        match self.file_map.get(filename) {
            None => {
                self.file_map.insert(filename.to_string(), File::empty());
                Ok(())
            }
            Some(_) => Err(FileError::FileNameAlreadyExists),
        }
    }

    pub fn open(&mut self, filename: &str, with_write: bool) -> Result<OpenedFile, FileError> {
        let mut file = match self.file_map.get_mut(filename) {
            Some(f) => f,
            None => {
                return Err(FileError::FileDoesNotExist);
            }
        };
        if with_write {
            file.is_opened_for_write = true;
        } else {
            file.is_opened_for_read += 1;
        }
        Ok(OpenedFile {
            file,
            read_pointer: 0,
        })
    }
}
