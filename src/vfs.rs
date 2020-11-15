use crate::alloc::borrow::ToOwned;
use crate::alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;

use crate::device_driver;
pub use num_traits::FromPrimitive;

device_driver!(
    unsynchronized VIRTUAL_FILE_SYSTEM: VFS = VFS::example_vfs()
);

pub fn open(filename: &str, with_write: bool) -> Result<OpenedFile, FileError> {
    let mut fs = VIRTUAL_FILE_SYSTEM.lock();
    fs.open(filename, with_write)
}

pub fn close(of: &mut OpenedFile) -> Result<(), FileError> {
    let mut fs = VIRTUAL_FILE_SYSTEM.lock();
    fs.close(of)
}

pub fn write(of: &OpenedFile, message: &[u8]) -> Result<(), FileError> {
    let mut fs = VIRTUAL_FILE_SYSTEM.lock();
    fs.write(of, message)
}

pub fn create_file(filename: &str) -> Result<(), FileError> {
    let mut fs = VIRTUAL_FILE_SYSTEM.lock();
    fs.create_file(filename)
}

pub fn delete_file(filename: &str) -> Result<(), FileError> {
    let mut fs = VIRTUAL_FILE_SYSTEM.lock();
    fs.delete_file(filename)
}

pub fn seek(
    of: &mut OpenedFile,
    difference: isize,
    seek_type: SeekType,
) -> Result<usize, FileError> {
    let mut fs = VIRTUAL_FILE_SYSTEM.lock();
    fs.seek(of, difference, seek_type)
}

pub fn read(of: &mut OpenedFile, length: usize) -> Result<ReadData, FileError> {
    let mut fs = VIRTUAL_FILE_SYSTEM.lock();
    let data = fs.read(of, length);

    if data.is_err() {
        return Err(data.err().unwrap());
    }

    let bytes = data.unwrap();

    Ok(ReadData {
        data: bytes.as_ptr(),
        len: bytes.len(),
    })
}

pub struct ReadData {
    pub data: *const u8,
    pub len: usize,
}

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum FileError {
    FileNameAlreadyExists,
    FileDoesNotExist,
    AttemptToCloseClosedFile,
    PositionOutOfBoundsOfFile,
    ModifyingWithoutWritePermission,
    ReadOnClosedFile,
    FileAlreadyOpenedForWrite,
    FileAlreadyOpenedForRead,
    CannotDeleteOpenedFile,
    CannotReadWriteOnlyFile,
    CannotSeekSpecialFile,
    CannotCloseSpecialFile,
}

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum SeekType {
    FromBeginning,
    FromCurrent,
    FromEnd,
}

pub struct File {
    pub data: Vec<u8>,
    pub is_opened_for_read: u16,
    pub is_opened_for_write: bool,
}

impl File {
    pub fn empty() -> Self {
        File {
            data: Vec::new(),
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

pub struct OpenedFile {
    filename: String,
    cursor: usize,
}

pub struct VFS {
    file_map: BTreeMap<String, File>,
}

impl Default for VFS {
    fn default() -> Self {
        Self::new()
    }
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
            data: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam volutpat posuere massa, quis feugiat diam consectetur eget. Quisque vitae feugiat odio. Pellentesque sed sem eu turpis aliquet lacinia. Nam facilisis finibus mi vitae dignissim. Praesent id nunc leo. Nulla non dapibus justo, quis sagittis est. Maecenas et lorem a nulla imperdiet facilisis ac sit amet nulla. Nulla facilisi. Fusce orci nibh, dapibus at rhoncus non, faucibus eget ipsum. Suspendisse potenti. Nunc tempor felis elit, rhoncus porta ante porttitor id. Ut viverra tincidunt feugiat. Curabitur enim elit, fringilla ac metus eget, vestibulum malesuada enim. Proin ac augue dignissim, egestas lacus eu, dictum eros. Suspendisse rutrum venenatis risus eleifend consectetur.".as_bytes().to_owned(),
            is_opened_for_read: 0,
            is_opened_for_write: false});
        vfs.file_map.insert("file2".to_string(), File{
            data: "Bee Movie Script - Dialogue Transcript According to all known laws of aviation, there is no way a bee should be able to fly. Its wings are too small to get its fat little body off the ground. The bee, of course, flies anyway because bees don't care what humans think is impossible. Yellow, black. Yellow, black. Yellow, black. Yellow, black. Ooh, black and yellow! Let's shake it up a little. Barry! Breakfast is ready! Ooming!".as_bytes().to_owned(),
            is_opened_for_read: 0,
            is_opened_for_write: false});
        vfs
    }

    // pub fn list_files(&self) -> Vec<String>{
    //     self.file_map.keys().copied().collect()
    // }

    pub fn create_file(&mut self, filename: &str) -> Result<(), FileError> {
        match self.file_map.get(filename) {
            None => {
                self.file_map.insert(filename.to_string(), File::empty());
                Ok(())
            }
            Some(_) => Err(FileError::FileNameAlreadyExists),
        }
    }

    pub fn delete_file(&mut self, filename: &str) -> Result<(), FileError> {
        match self.file_map.get(filename) {
            Some(f) => {
                if f.is_opened_for_write || f.is_opened_for_read > 0 {
                    return Err(FileError::CannotDeleteOpenedFile);
                }
                self.file_map.remove(filename);
                Ok(())
            }
            None => Err(FileError::FileDoesNotExist),
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
            if file.is_opened_for_write {
                return Err(FileError::FileAlreadyOpenedForWrite);
            } else if file.is_opened_for_read > 0 {
                return Err(FileError::FileAlreadyOpenedForRead);
            } else {
                file.is_opened_for_write = true;
            }
        } else {
            if file.is_opened_for_write {
                return Err(FileError::FileAlreadyOpenedForWrite);
            }
            file.is_opened_for_read += 1;
        }
        Ok(OpenedFile {
            filename: String::from(filename),
            cursor: 0,
        })
    }
    pub fn read(&mut self, of: &mut OpenedFile, length: usize) -> Result<&[u8], FileError> {
        let file = match self.file_map.get(&of.filename) {
            Some(f) => {
                if f.is_opened_for_read > 0 || f.is_opened_for_write {
                    f
                } else {
                    return Err(FileError::ReadOnClosedFile);
                }
            }
            None => {
                return Err(FileError::FileDoesNotExist);
            }
        };
        let file_len = file.data.len();
        let end_of_read = if file_len < of.cursor + length {
            file_len
        } else {
            of.cursor + length
        };
        let result = &file.data[of.cursor..end_of_read];
        of.cursor = end_of_read;
        Ok(result)
    }

    pub fn write(&mut self, of: &OpenedFile, message: &[u8]) -> Result<(), FileError> {
        let file = match self.file_map.get_mut(&of.filename) {
            Some(f) => {
                if f.is_opened_for_write {
                    f
                } else {
                    return Err(FileError::ModifyingWithoutWritePermission);
                }
            }
            None => {
                return Err(FileError::FileDoesNotExist);
            }
        };
        let split_off = file.data.split_off(of.cursor);
        file.data.extend_from_slice(message);
        file.data.extend(split_off);
        Ok(())
    }

    pub fn close(&mut self, of: &mut OpenedFile) -> Result<(), FileError> {
        let mut file = match self.file_map.get_mut(&of.filename) {
            Some(f) => f,
            None => {
                return Err(FileError::FileDoesNotExist);
            }
        };
        if file.is_opened_for_write {
            file.is_opened_for_write = false;
        } else {
            if file.is_opened_for_read == 0 {
                return Err(FileError::AttemptToCloseClosedFile);
            }
            file.is_opened_for_read -= 1;
        }
        Ok(())
    }

    pub fn seek(
        &mut self,
        of: &mut OpenedFile,
        difference: isize,
        seek_type: SeekType,
    ) -> Result<usize, FileError> {
        let file = match self.file_map.get_mut(&of.filename) {
            Some(f) => {
                if f.is_opened_for_write {
                    f
                } else {
                    return Err(FileError::ModifyingWithoutWritePermission);
                }
            }
            None => {
                return Err(FileError::FileDoesNotExist);
            }
        };
        let size = file.data.len();
        match seek_type {
            SeekType::FromBeginning => {
                if difference < 0 {
                    of.cursor = 0;
                } else {
                    of.cursor = core::cmp::min(difference as usize, size);
                }
            }
            SeekType::FromCurrent => {
                if difference < 0 {
                    of.cursor = core::cmp::max(
                        of.cursor
                            .checked_sub(-difference as usize)
                            .unwrap_or_else(|| 0usize),
                        0usize,
                    );
                } else {
                    of.cursor = core::cmp::min(
                        of.cursor
                            .checked_add(difference as usize)
                            .unwrap_or_else(|| size),
                        size,
                    );
                }
            }
            SeekType::FromEnd => {
                if difference < 0 {
                    of.cursor = size;
                } else {
                    of.cursor = core::cmp::max(size - (difference as usize), 0);
                }
            }
        }
        Ok(of.cursor)
    }
}
