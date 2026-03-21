use core::ffi::{CStr, c_char};

use crate::system::syscalls::types::{SyscallError, SyscallFrame};
use crate::system::vfs::{FdKind, VFSError};
use crate::system::{self};
use crate::{error, print};

pub fn read(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let buf = frame.rsi as *mut u8;
    let len = frame.rdx as usize;

    let result =
        system::proc::with_fd_table(|table| match table.get_mut(fd)? {
            FdKind::File(file) => {
                let slice =
                    unsafe { core::slice::from_raw_parts_mut(buf, len) };
                file.read(slice)
            },
            _ => {
                error!("read syscall: fd {} is not readable", fd);
                Err(VFSError::PermissionDenied)
            },
        });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(SyscallError::NotFound)
    }
}

pub fn write(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let buf = frame.rsi as *const u8;
    let len = frame.rdx as usize;

    let result = system::proc::with_fd_table(|table| match table.get(fd)? {
        FdKind::Stdout | FdKind::Stderr => {
            for i in 0..len {
                let byte = unsafe { *buf.add(i) };
                print!("{}", byte as char);
            }
            Ok(len)
        },
        _ => {
            error!("write syscall: fd {} is not writable", fd);
            Err(VFSError::PermissionDenied)
        },
    });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(SyscallError::NotFound)
    }
}

pub fn open(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let path =
        unsafe { CStr::from_ptr(frame.rdi as *const c_char).to_str().unwrap() };
    let flags = frame.rsi as u32;
    let _mode = frame.rdx as usize;

    // TODO: handle directory
    match system::vfs::open(path, flags) {
        Ok(file) => {
            let result = system::proc::with_fd_table(|table| {
                table.alloc(FdKind::File(file))
            });
            Ok(result.map(|fd| fd as u64).unwrap_or(u64::MAX))
        },
        Err(_) => Err(SyscallError::NotFound),
    }
}

pub fn close(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let result = system::proc::with_fd_table(|table| table.close(fd));
    if result.is_ok() { Ok(0) } else { Err(SyscallError::NotFound) }
}

pub fn seek(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let offset = frame.rsi as i64;
    let whence = frame.rdx as u32;

    let result =
        system::proc::with_fd_table(|table| match table.get_mut(fd)? {
            FdKind::File(file) => file.seek(match whence {
                0 => system::vfs::VFSSeek::Start(offset as usize),
                1 => system::vfs::VFSSeek::Current(offset as usize),
                2 => system::vfs::VFSSeek::End(offset as usize),
                _ => return Err(VFSError::InvalidSeek),
            }),
            _ => {
                error!("seek syscall: fd {} is not seekable", fd);
                Err(VFSError::PermissionDenied)
            },
        });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(SyscallError::NotFound)
    }
}
