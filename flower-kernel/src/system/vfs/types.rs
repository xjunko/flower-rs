use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
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
pub enum VFSFileType {
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
    pub typ: VFSFileType,
    pub size: usize,
    pub last_modified: usize,
    pub owner_id: usize,
    pub group_id: usize,
    pub permissions: VFSPermissions,
}

#[derive(Debug, Clone, Copy)]
pub enum VFSSeek {
    Start(usize),
    Current(usize),
    End(usize),
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

pub trait VFSFile: Send + Sync {
    /// reads data into the given buffer and returns the number of bytes read
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize>;

    /// writes data from the given buffer and returns the number of bytes written
    fn write(&self, buf: &mut [u8]) -> VFSResult<usize>;

    /// seeks to the given position and returns the new position
    fn seek(&mut self, pos: VFSSeek) -> VFSResult<usize>;

    /// maps the file into memory and returns a pointer to the mapped region
    fn mmap(&self, len: usize, prot: c_int, flags: c_int)
    -> VFSResult<*mut u8>;

    /// gets the info for the file
    fn metadata(&self) -> VFSResult<VFSMetadata>;
}

pub trait VFSImplementation: Send + Sync {
    /// opens the file
    fn open(&self, path: &str, flags: u32) -> VFSResult<Box<dyn VFSFile>>;

    /// gets the info for the file
    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata>;

    /// checks if the file exists
    fn exists(&self, path: &str) -> bool { self.metadata(path).is_ok() }
}
