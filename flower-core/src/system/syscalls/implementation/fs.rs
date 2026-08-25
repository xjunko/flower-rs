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

use core::ffi::{CStr, c_char};

use flower_uapi::structs::FileStat;

use crate::system::syscalls::types::{SyscallError, SyscallFrame};
use crate::system::vfs::error::VfsError;
use crate::system::vfs::file::{OpenFlags, Whence};
use crate::system::vfs::perm::Credentials;
use crate::system::{self, ToSyscallError};

pub fn open(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let path =
        unsafe { CStr::from_ptr(frame.rdi as *const c_char).to_str().unwrap() };
    let flags = frame.rsi as u32;
    let _mode = frame.rdx as usize;

    // TODO: handle directory
    if let Ok(file) =
        system::vfs::open(path, OpenFlags::from_bits(flags), Credentials::ROOT)
    {
        let result = system::proc::with_fd_table(|table| table.install(file));
        Ok(result.map(|fd| fd as u64).unwrap_or(u64::MAX))
    } else {
        Err(SyscallError::IOError)
    }
}

pub fn read(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let buf = frame.rsi as *mut u8;
    let len = frame.rdx as usize;

    let result = system::proc::with_fd_table(|table| match table.get(fd) {
        Ok(file) => {
            let slice = unsafe { core::slice::from_raw_parts_mut(buf, len) };
            file.read(slice)
        },
        Err(_) => {
            log::error!("read syscall: fd {} is not readable", fd);
            Err(system::vfs::error::VfsError::PermissionDenied)
        },
    });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(result.err().unwrap().to_syscall_error())
    }
}

pub fn write(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let _buf = frame.rsi as *mut u8;
    let _len = frame.rdx as usize;

    let result: Result<u64, VfsError> =
        system::proc::with_fd_table(|table| match table.get(fd) {
            Ok(file) => {
                let slice = unsafe { core::slice::from_raw_parts(_buf, _len) };
                file.write(slice).map(|written| written as u64)
            },
            Err(_) => {
                log::error!("write syscall: fd {} is not writable", fd);
                Err(VfsError::Unsupported)
            },
        });

    if let Ok(result) = result {
        Ok(result)
    } else {
        Err(result.err().unwrap().to_syscall_error())
    }
}

pub fn close(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let result = system::proc::with_fd_table(|table| table.close(fd));
    if result.is_ok() { Ok(0) } else { Err(SyscallError::BadFileDescriptor) }
}

pub fn seek(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let offset = frame.rsi as i64;
    let whence = frame.rdx as u32;

    let result = system::proc::with_fd_table(|table| match table.get(fd) {
        Ok(file) => file.seek(
            offset,
            match whence {
                0 => Whence::Start,
                1 => Whence::Current,
                2 => Whence::End,
                _ => {
                    return Err(VfsError::InvalidArgument);
                },
            },
        ),
        Err(_) => {
            log::error!("seek syscall: fd {} is not seekable", fd);
            Err(VfsError::PermissionDenied)
        },
    });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(result.err().unwrap().to_syscall_error())
    }
}

pub fn stat(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let fd = frame.rdi as usize;
    let stat_buf = frame.rsi as *mut FileStat;

    log::debug!("stat syscall: fd={}, stat_buf.size={:?}", fd, stat_buf);

    let result = system::proc::with_fd_table(|table| match table.get(fd) {
        Ok(file) => {
            let stat = file.metadata()?;
            unsafe {
                (*stat_buf).st_mode = 0; // TODO: set mode
                (*stat_buf).st_dev = 0; // TODO: set device id
                (*stat_buf).st_size = stat.size as u64;
            }
            Ok(0)
        },
        Err(_) => {
            log::error!("stat syscall: fd {} is not statable", fd);
            Err(VfsError::PermissionDenied)
        },
    });

    if result.is_ok() {
        Ok(0)
    } else {
        Err(result.err().unwrap().to_syscall_error())
    }
}
