use alloc::string::String;
use core::error::Error;
use core::fmt::{Display, Formatter};

use crate::std;

#[derive(Debug)]
pub enum FileError {
    FileNotFound,
    FileReadError,
    FileWriteError,
    FileInvalid,
}

impl Error for FileError {}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), core::fmt::Error> {
        match self {
            FileError::FileNotFound => write!(f, "File not found"),
            FileError::FileReadError => write!(f, "Failed to read from file"),
            FileError::FileWriteError => write!(f, "Failed to write to file"),
            FileError::FileInvalid => write!(f, "Invalid file descriptor"),
        }
    }
}

pub struct File {
    fd: u64,
}

impl File {
    pub fn open(path: String) -> Result<Self, FileError> {
        let fd = std::open(path.as_bytes(), 0, 0);
        if fd < 0 {
            Err(FileError::FileNotFound)
        } else {
            Ok(Self { fd: fd as u64 })
        }
    }

    pub fn close(&self) -> Result<(), FileError> {
        if std::close(self.fd) < 0 {
            Err(FileError::FileInvalid)
        } else {
            Ok(())
        }
    }
}

impl File {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, FileError> {
        let result = std::read(self.fd, buf);
        if result < 0 {
            Err(FileError::FileReadError)
        } else {
            Ok(result as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, FileError> {
        let result = std::write(self.fd, buf);
        if result < 0 {
            Err(FileError::FileWriteError)
        } else {
            Ok(result as usize)
        }
    }
}
