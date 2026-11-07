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

use flower_abi::fs;
use flower_abi::structs::FileStat;
use flower_abi::system::WinSize;

use crate::arch::addr::KVirtAddr;
use crate::system::posix::errno::{KError, KResult};
use crate::system::syscalls::types::{SyscallFrame, SyscallReturnCode};
use crate::system::vfs::{FdKind, FileLike, Whence};
use crate::system::{self};
use crate::{arch, print};

pub fn open(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let path =
        unsafe { CStr::from_ptr(frame.rdi as *const c_char).to_str().unwrap() };
    let flags = frame.rsi as u32;
    let _mode = frame.rdx as usize;

    // TODO: handle directory
    match system::vfs::open(path, flags) {
        Ok(file) => match file {
            FileLike::File(f) => {
                let result = system::proc::with_fd_table(|table| {
                    table.alloc(FdKind::File(f))
                });
                Ok(result.map(|fd| fd as u64).unwrap_or(u64::MAX))
            },
            _ => {
                log::error!("open syscall: {} is not a regular file", path);
                Err(KError::EIO)
            },
        },
        Err(e) => Err(e),
    }
}

pub fn read(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
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
            FdKind::Stdin => {
                unimplemented!("read from stdin");
            },
            _ => {
                log::error!("read syscall: fd {} is not readable", fd);
                Err(KError::EPERM)
            },
        });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(result.err().unwrap())
    }
}

pub fn read_v(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let iov = frame.rsi as *const fs::Iovec;
    let iovcnt = frame.rdx as usize;

    if iovcnt > fs::IOV_MAX {
        return Err(KError::EINVAL);
    }

    let result = system::proc::with_fd_table(|table| {
        let mut read_total = 0usize;
        match table.get_mut(fd)? {
            FdKind::File(file) => {
                for index in 0..iovcnt {
                    let entry = unsafe { &*iov.add(index) };
                    let slice = unsafe {
                        core::slice::from_raw_parts_mut(
                            entry.base as *mut u8,
                            entry.len,
                        )
                    };
                    read_total = read_total
                        .checked_add(file.read(slice)?)
                        .ok_or(KError::EIO)?;
                }
                Ok(read_total)
            },
            _ => Err(KError::EBADFD),
        }
    });

    result.map(|count| count as u64)
}

pub fn write(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let buf = frame.rsi as *mut u8;
    let len = frame.rdx as usize;

    let result = system::proc::with_fd_table(|table| match table.get(fd)? {
        FdKind::Stdout | FdKind::Stderr => {
            for i in 0..len {
                let byte = unsafe { *buf.add(i) };
                print!("{}", byte as char);
            }
            Ok(len)
        },
        FdKind::File(file) => {
            let slice = unsafe { core::slice::from_raw_parts_mut(buf, len) };
            let written = file.write(slice)?;
            Ok(written)
        },
        _ => {
            log::error!("write syscall: fd {} is not writable", fd);
            Err(KError::EPERM)
        },
    });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(result.expect_err("error not found while erroring"))
    }
}

pub fn write_v(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let iov = frame.rsi as *const fs::Iovec;
    let iovcnt = frame.rdx as usize;

    if iovcnt > fs::IOV_MAX {
        return Err(KError::EINVAL);
    }

    let result = system::proc::with_fd_table(|table| {
        let mut written = 0usize;
        match table.get_mut(fd)? {
            FdKind::Stdout | FdKind::Stderr => {
                for index in 0..iovcnt {
                    let entry = unsafe { &*iov.add(index) };
                    for offset in 0..entry.len {
                        let byte = unsafe { *entry.base.add(offset) };
                        print!("{}", byte as char);
                    }
                    written =
                        written.checked_add(entry.len).ok_or(KError::EIO)?;
                }
                Ok(written)
            },
            FdKind::File(file) => {
                for index in 0..iovcnt {
                    let entry = unsafe { &*iov.add(index) };
                    let slice = unsafe {
                        core::slice::from_raw_parts_mut(
                            entry.base as *mut u8,
                            entry.len,
                        )
                    };
                    written = written
                        .checked_add(file.write(slice)?)
                        .ok_or(KError::EIO)?;
                }
                Ok(written)
            },
            _ => Err(KError::EBADFD),
        }
    });

    result.map(|count| count as u64)
}

pub fn close(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;

    let result = system::proc::with_fd_table(|table| match table.get(fd)? {
        // TODO: musl closes these
        FdKind::Stdout | FdKind::Stderr => {
            log::trace!("we're not closing stdout/stderr");
            Ok(())
        },
        _ => table.close(fd),
    });

    if result.is_ok() { Ok(0) } else { Err(KError::EBADF) }
}

pub fn seek(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let offset = frame.rsi as i64;
    let whence = frame.rdx as u32;

    let result =
        system::proc::with_fd_table(|table| match table.get_mut(fd)? {
            FdKind::File(file) => file.seek(
                offset,
                match whence {
                    0 => Whence::Start,
                    1 => Whence::Current,
                    2 => Whence::End,
                    _ => return Err(KError::EINVAL),
                },
            ),
            FdKind::Stdin | FdKind::Stdout | FdKind::Stderr => {
                Ok(0) // HACK: noop
            },
            // _ => {
            //     log::error!("seek syscall: fd {} is not seekable", fd);
            //     Err(VFSError::PermissionDenied)
            // },
        });

    if let Ok(result) = result {
        Ok(result as u64)
    } else {
        Err(result.expect_err("error not found while erroring"))
    }
}

pub fn stat(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let stat_buf = frame.rsi as *mut FileStat;

    log::debug!("stat syscall: fd={}, stat_buf.size={:?}", fd, stat_buf);

    let result = system::proc::with_fd_table(|table| match table.get(fd)? {
        FdKind::File(file) => {
            let stat = file.stat()?;
            unsafe {
                (*stat_buf).st_mode = 0; // TODO: set mode
                (*stat_buf).st_dev = 0; // TODO: set device id
                (*stat_buf).st_size = stat.size as u64;
            }
            Ok(0)
        },
        _ => {
            log::error!("stat syscall: fd {} is not statable", fd);
            Err(KError::EPERM)
        },
    });

    if result.is_ok() {
        Ok(0)
    } else {
        Err(result.expect_err("err not found, while erroring."))
    }
}

pub fn ioctl(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let request = frame.rsi;
    let argp = frame.rdx as *mut u8;

    log::debug!(
        "ioctl syscall: fd={}, request={:#x}, argp={:?}",
        fd,
        request,
        argp
    );

    let result =
        system::proc::with_fd_table(|table| match table.get_mut(fd)? {
            FdKind::File(file) => file.ioctl(request, argp as usize),
            FdKind::Stdout => match request as usize {
                flower_abi::system::TIOCGWINSZ => {
                    let win_size = WinSize {
                        ws_row: 0,
                        ws_col: 0,
                        ws_xpixel: 0,
                        ws_ypixel: 0,
                    };

                    arch::user::copy_to_user(
                        KVirtAddr::new(argp as u64),
                        win_size,
                    )
                    .expect("failed to copy win_size to libc");
                    Ok(0)
                },
                _ => Err(KError::ENOTTY),
            },
            _ => {
                log::error!("ioctl syscall: fd {} is not ioctlable", fd);
                Err(KError::ENOTTY)
            },
        });

    if result.is_ok() { Ok(0) } else { Err(KError::ENOTTY) }
}

pub fn access(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let path =
        unsafe { CStr::from_ptr(frame.rdi as *const c_char).to_str().unwrap() };
    let mode = frame.rsi as u32;

    log::debug!("access syscall: path={}, mode={:#x}", path, mode);

    if system::vfs::open(path, 0).is_ok() {
        return Ok(0); // TODO: whatever man, you can have at it idc
    }

    Err(KError::ENOENT)
}

pub fn pipe(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let pipefd_ptr = frame.rdi as *mut fs::PipeFd;
    log::trace!("pipe syscall: pipefd_ptr={:?}", pipefd_ptr);
    Err(KError::ENOSYS)
}

pub fn umask(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let mask = frame.rdi as u32;
    log::trace!("umask syscall: mask={:#o}", mask);
    Err(KError::ENOSYS)
}
