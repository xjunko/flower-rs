/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

use alloc::string::String;
use core::error::Error;
use core::fmt::{Display, Formatter};

use flower_uapi::mmap::{MAP_SHARED, PROT_READ, PROT_WRITE};
use flower_uapi::structs::FileStat;

use crate::sys::{fs, kernel};

#[derive(Debug)]
pub enum FileError {
    FileNotFound,
    FileReadError,
    FileWriteError,
    FileMmapError,
    FileInvalid,
}

impl Error for FileError {}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), core::fmt::Error> {
        match self {
            FileError::FileNotFound => write!(f, "File not found"),
            FileError::FileReadError => write!(f, "Failed to read from file"),
            FileError::FileWriteError => write!(f, "Failed to write to file"),
            FileError::FileMmapError => write!(f, "Failed to mmap file"),
            FileError::FileInvalid => write!(f, "Invalid file descriptor"),
        }
    }
}

pub struct FileMetadata {
    pub size: usize,
}

impl From<FileStat> for FileMetadata {
    fn from(stat: FileStat) -> Self { Self { size: stat.st_size as usize } }
}

pub struct File {
    fd: u64,
}

impl File {
    pub fn fd(&self) -> u64 { self.fd }

    pub fn open(path: String, flags: u32) -> Result<Self, FileError> {
        let fd = fs::open(path.as_ptr(), path.len(), flags as u64, 0);
        if fd < 0 {
            Err(FileError::FileNotFound)
        } else {
            Ok(Self { fd: fd as u64 })
        }
    }

    // drop() will call this.
    fn close(&mut self) -> Result<(), FileError> {
        if fs::close(self.fd) < 0 {
            Err(FileError::FileInvalid)
        } else {
            Ok(())
        }
    }
}

impl File {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, FileError> {
        let result = fs::read(self.fd, buf.as_mut_ptr(), buf.len());
        if result < 0 {
            Err(FileError::FileReadError)
        } else {
            Ok(result as usize)
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, FileError> {
        let result = fs::write(self.fd, buf.as_ptr(), buf.len());
        if result < 0 {
            Err(FileError::FileWriteError)
        } else {
            Ok(result as usize)
        }
    }

    pub fn metadata(&self) -> Result<FileMetadata, FileError> {
        let mut stat = FileStat::default();

        if unsafe { fs::metadata(self.fd, &mut stat) } < 0 {
            Err(FileError::FileInvalid)
        } else {
            Ok(FileMetadata::from(stat))
        }
    }

    pub fn mmap(&self, length: usize) -> Result<*mut u8, FileError> {
        let addr = kernel::mmap(
            core::ptr::null_mut(),
            length,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            self.fd,
            0,
        );
        if addr.is_null() { Err(FileError::FileMmapError) } else { Ok(addr) }
    }
}

impl Drop for File {
    fn drop(&mut self) { self.close().expect("Failed to close file in Drop"); }
}
