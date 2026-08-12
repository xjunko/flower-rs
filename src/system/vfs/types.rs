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

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_int;

use crate::system::ToSyscallError;
use crate::system::syscalls::SyscallError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VFSPermission {
    Read = 0b100,
    Write = 0b010,
    Execute = 0b001,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VFSPermissionShift {
    Owner = 6,
    Group = 3,
    Other = 0,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VFSPermissions {
    bits: u16,
}

impl VFSPermissions {
    pub fn new() -> Self { Self { bits: 0 } }

    pub fn from_unix(perm: usize) -> Self {
        Self { bits: (perm & 0o777) as u16 }
    }

    pub fn has(&self, perm: VFSPermission, shift: VFSPermissionShift) -> bool {
        (self.bits & ((perm as u16) << (shift as u16))) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VFSMetadataFileType {
    File,
    Directory,
    Device,
    Symlink,
    Pipe,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct VFSMetadata {
    pub name: String,
    pub size: usize,
    pub typ: VFSMetadataFileType,
    pub last_modified: usize,
    pub owner_id: usize,
    pub group_id: usize,
    pub permissions: VFSPermissions,
}

#[derive(Debug, Clone, Copy)]
pub enum VFSWhence {
    Start,
    Current,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VFSError {
    NotFound,
    AlreadyExists,
    InvalidSeek,
    PermissionDenied,
    NoSpace,
    IOError,
    Unsupported,
    Unknown(String),
}

impl ToSyscallError for VFSError {
    fn to_syscall_error(&self) -> SyscallError {
        match self {
            Self::NotFound => SyscallError::NoSuchFile,
            Self::InvalidSeek => SyscallError::InvalidArgument,
            Self::PermissionDenied => SyscallError::NoPermission,
            Self::IOError => SyscallError::IOError,
            _ => SyscallError::Other(format!("Unhandled VFSError: {:?}", self)),
        }
    }
}

pub type VFSResult<T> = Result<T, VFSError>;

pub enum VFSFilelike {
    File(Box<dyn VFSFile>),
    Directory(Box<dyn VFSDirectory>),
}

pub trait VFSFile: Send + Sync {
    /// gets the name of the file
    fn name(&self) -> VFSResult<String>;

    /// reads data into the given buffer and returns the number of bytes read
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize>;

    /// writes data from the given buffer and returns the number of bytes written
    fn write(&self, buf: &mut [u8]) -> VFSResult<usize>;

    /// seeks to the given position and returns the new position
    fn seek(&mut self, offset: i64, pos: VFSWhence) -> VFSResult<usize>;

    /// maps the file into memory and returns a pointer to the mapped region
    fn mmap(
        &self,
        len: usize,
        prot: c_int,
        flags: c_int,
        offset: u64,
    ) -> VFSResult<*mut u8>;

    /// gets the info for the file
    fn metadata(&self) -> VFSResult<VFSMetadata>;

    /// changes the permissions of the file
    fn chmod(&self, _mode: u32) -> VFSResult<()> { Err(VFSError::Unsupported) }

    /// changes the owner of the file
    fn chown(&self, _uid: u32, _gid: u32) -> VFSResult<()> {
        Err(VFSError::Unsupported)
    }
}

pub trait VFSDirectory: Send + Sync {
    /// get the name of the directory
    fn name(&self) -> VFSResult<String>;

    /// get the contents of the directory
    fn contents(&self) -> VFSResult<Vec<VFSFilelike>>;

    /// deletes a file in the directory
    fn delete(&self, name: &str) -> VFSResult<()>;

    /// changes the permissions of the directory
    fn chmod(&self, _mode: u32) -> VFSResult<()> { Err(VFSError::Unsupported) }

    /// changes the owner of the directory
    fn chown(&self, _uid: u32, _gid: u32) -> VFSResult<()> {
        Err(VFSError::Unsupported)
    }
}

pub trait VFSImplementation: Send + Sync {
    /// initializes the filesystem
    fn initialize(&mut self) -> VFSResult<()>;

    /// opens the file
    fn open(&self, path: &str, flags: u32) -> VFSResult<VFSFilelike>;

    /// gets the info for the file
    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata>;

    /// checks if the file exists
    fn exists(&self, path: &str) -> bool { self.metadata(path).is_ok() }
}
