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
use core::ffi::{CStr, c_char};

use flower_abi::fs::{self, O_CLOEXEC, O_NONBLOCK};
use flower_abi::structs::FileStat;
use flower_abi::system::WinSize;

use crate::arch::addr::KVirtAddr;
use crate::system::mem::vma::Shmem;
use crate::system::posix::errno::{KError, KResult};
use crate::system::syscalls::types::{SyscallFrame, SyscallReturnCode};
use crate::system::vfs::pipe::KPipeEnd;
use crate::system::vfs::{FdKind, FileType, KFile, Stat, Whence};
use crate::system::{self};
use crate::{arch, print};

fn alloc_fd(kind: FdKind) -> KResult<usize> {
    system::proc::with_fd_table(|table| table.alloc(kind))
}

fn fd_kind(fd: usize) -> KResult<FdKind> { system::proc::fd(fd) }

fn read_pipe(pipe: &KPipeEnd, buf: &mut [u8]) -> KResult<usize> {
    loop {
        match pipe.read(buf) {
            Err(KError::EAGAIN) if !pipe.is_nonblock() => {
                system::proc::sleep(1_000_000);
            },
            result => return result,
        }
    }
}

fn write_pipe(pipe: &KPipeEnd, buf: &[u8]) -> KResult<usize> {
    loop {
        match pipe.write(buf) {
            Err(KError::EAGAIN) if !pipe.is_nonblock() => {
                system::proc::sleep(1_000_000);
            },
            result => return result,
        }
    }
}

fn read_into(kind: &FdKind, buf: &mut [u8]) -> KResult<usize> {
    match kind {
        FdKind::File(file) => file.read(buf),
        FdKind::Pipe(pipe) => read_pipe(pipe, buf),
        FdKind::Stdin => Err(KError::ENOSYS),
        _ => Err(KError::EBADF),
    }
}

fn write_from(kind: &FdKind, buf: &[u8]) -> KResult<usize> {
    match kind {
        FdKind::File(file) => file.write(buf),
        FdKind::Pipe(pipe) => write_pipe(pipe, buf),
        FdKind::Stdout | FdKind::Stderr => {
            for byte in buf {
                print!("{}", *byte as char);
            }
            Ok(buf.len())
        },
        _ => Err(KError::EBADF),
    }
}

pub fn open(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let path = unsafe { CStr::from_ptr(frame.rdi as *const c_char) }
        .to_str()
        .map_err(|_| KError::EINVAL)?;
    let flags = frame.rsi as u32;

    let fd = match system::vfs::open(path, flags)? {
        KFile::File(file) => alloc_fd(FdKind::File(file))?,
        KFile::Shmem(shmem) => alloc_fd(FdKind::Shmem(shmem))?,
        KFile::Directory(_) => return Err(KError::EISDIR),
    };

    Ok(fd as u64)
}

pub fn read(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let len = frame.rdx as usize;

    if len == 0 {
        return Ok(0);
    }

    let kind = fd_kind(fd)?;
    let buf =
        unsafe { core::slice::from_raw_parts_mut(frame.rsi as *mut u8, len) };

    read_into(&kind, buf).map(|count| count as u64)
}

pub fn read_v(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let iov = frame.rsi as *const fs::Iovec;
    let iovcnt = frame.rdx as usize;

    if iovcnt > fs::IOV_MAX {
        return Err(KError::EINVAL);
    }

    let kind = fd_kind(fd)?;
    let mut total = 0usize;

    for index in 0..iovcnt {
        let entry = unsafe { &*iov.add(index) };

        if entry.len == 0 {
            continue;
        }

        let slice = unsafe {
            core::slice::from_raw_parts_mut(entry.base as *mut u8, entry.len)
        };

        total =
            total.checked_add(read_into(&kind, slice)?).ok_or(KError::EIO)?;
    }

    Ok(total as u64)
}

pub fn write(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let len = frame.rdx as usize;

    if len == 0 {
        return Ok(0);
    }

    let kind = fd_kind(fd)?;
    let buf =
        unsafe { core::slice::from_raw_parts(frame.rsi as *const u8, len) };

    write_from(&kind, buf).map(|count| count as u64)
}

pub fn write_v(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let iov = frame.rsi as *const fs::Iovec;
    let iovcnt = frame.rdx as usize;

    if iovcnt > fs::IOV_MAX {
        return Err(KError::EINVAL);
    }

    let kind = fd_kind(fd)?;
    let mut total = 0usize;

    for index in 0..iovcnt {
        let entry = unsafe { &*iov.add(index) };

        if entry.len == 0 {
            continue;
        }

        let slice =
            unsafe { core::slice::from_raw_parts(entry.base, entry.len) };

        total =
            total.checked_add(write_from(&kind, slice)?).ok_or(KError::EIO)?;
    }

    Ok(total as u64)
}

pub fn close(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    system::proc::with_fd_table(|table| table.close(fd))?;
    Ok(0)
}

pub fn seek(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let offset = frame.rsi as i64;
    let whence = match frame.rdx as u32 {
        0 => Whence::Start,
        1 => Whence::Current,
        2 => Whence::End,
        _ => return Err(KError::EINVAL),
    };

    let result = match fd_kind(fd)? {
        FdKind::File(file) => file.seek(offset, whence),
        FdKind::Pipe(_) | FdKind::Shmem(_) => Err(KError::ESPIPE),
        FdKind::Stdin | FdKind::Stdout | FdKind::Stderr => Ok(0),
    };

    result.map(|position| position as u64)
}

pub fn stat(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let stat_buf = frame.rsi as *mut FileStat;

    let stat = match fd_kind(fd)? {
        FdKind::File(file) => file.stat()?,
        FdKind::Shmem(shmem) => Stat {
            name: String::from("shmem"),
            size: shmem.len(),
            typ: FileType::File,
            last_modified: 0,
        },
        FdKind::Pipe(_) => Stat {
            name: String::from("pipe"),
            size: 0,
            typ: FileType::Pipe,
            last_modified: 0,
        },
        _ => return Err(KError::EBADF),
    };

    let mode = match stat.typ {
        FileType::File => 0o100000,
        FileType::Directory => 0o040000,
        FileType::Pipe => 0o010000,
        _ => 0,
    };

    unsafe {
        (*stat_buf).st_mode = mode;
        (*stat_buf).st_dev = 0;
        (*stat_buf).st_size = stat.size as u64;
    }

    Ok(0)
}

pub fn ioctl(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let request = frame.rsi;
    let argp = frame.rdx as *mut u8;

    match fd_kind(fd)? {
        FdKind::File(file) => {
            file.ioctl(request, argp as usize)?;
        },
        FdKind::Stdout => match request as usize {
            flower_abi::system::TIOCGWINSZ => {
                let win_size = WinSize {
                    ws_row: 0,
                    ws_col: 0,
                    ws_xpixel: 0,
                    ws_ypixel: 0,
                };

                arch::user::copy_to_user(KVirtAddr::new(argp as u64), win_size)
                    .expect("failed to copy win_size to libc");
            },
            _ => return Err(KError::ENOTTY),
        },
        _ => return Err(KError::ENOTTY),
    }

    Ok(0)
}

pub fn access(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let path = unsafe { CStr::from_ptr(frame.rdi as *const c_char) }
        .to_str()
        .map_err(|_| KError::EINVAL)?;

    system::vfs::open(path, 0)?;
    Ok(0)
}

pub fn pipe(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    create_pipe(frame, 0)
}

pub fn pipe2(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    create_pipe(frame, frame.rsi as u32)
}

fn create_pipe(
    frame: &mut SyscallFrame,
    flags: u32,
) -> KResult<SyscallReturnCode> {
    if flags & !(O_NONBLOCK | O_CLOEXEC) != 0 {
        return Err(KError::EINVAL);
    }

    let (read, write) =
        crate::system::vfs::pipe::create(flags & O_NONBLOCK != 0);
    let read_fd = alloc_fd(FdKind::Pipe(read))?;

    let write_fd = match alloc_fd(FdKind::Pipe(write)) {
        Ok(fd) => fd,
        Err(e) => {
            let _ = system::proc::with_fd_table(|table| table.close(read_fd));
            return Err(e);
        },
    };

    let fds = fs::PipeFd { read_fd: read_fd as i32, write_fd: write_fd as i32 };

    if let Err(e) = arch::user::copy_to_user(KVirtAddr::new(frame.rdi), fds) {
        let _ = system::proc::with_fd_table(|table| {
            table.close(read_fd)?;
            table.close(write_fd)
        });
        return Err(e);
    }

    Ok(0)
}

pub fn ftruncate(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let len = frame.rsi as usize;

    match fd_kind(fd)? {
        FdKind::Shmem(shmem) => {
            shmem.resize(len).map_err(|_| KError::ENOMEM)?;
        },
        _ => return Err(KError::EINVAL),
    }

    Ok(0)
}

pub fn unlink(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let path = unsafe { CStr::from_ptr(frame.rdi as *const c_char) }
        .to_str()
        .map_err(|_| KError::EINVAL)?;

    system::vfs::unlink(path)?;
    Ok(0)
}

pub fn memfd_create(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let name = frame.rdi as *const c_char;

    if name.is_null() {
        return Err(KError::EFAULT);
    }

    let _name = unsafe { CStr::from_ptr(name) };

    alloc_fd(FdKind::Shmem(Shmem::new(0))).map(|fd| fd as u64)
}

pub fn dup(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let kind = fd_kind(frame.rdi as usize)?;
    alloc_fd(kind).map(|fd| fd as u64)
}

pub fn dup2(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let old = frame.rdi as usize;
    let new = frame.rsi as i64;

    if new < 0 || new as usize >= crate::system::vfs::MAX_FDS {
        return Err(KError::EBADF);
    }

    let new = new as usize;

    if old == new {
        fd_kind(old)?;
        return Ok(new as u64);
    }

    let kind = fd_kind(old)?;
    system::proc::with_fd_table(|table| table.set(new, kind))?;

    Ok(new as u64)
}

pub fn fcntl(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let fd = frame.rdi as usize;
    let command = frame.rsi;
    let argument = frame.rdx;

    match command {
        // fd flags are not tracked, so fds always survive exec (no CLOEXEC).
        fs::F_GETFD => Ok(0),
        fs::F_SETFD => Ok(0),
        fs::F_GETFL => {
            let flags = match fd_kind(fd)? {
                FdKind::Pipe(pipe) if pipe.is_nonblock() => O_NONBLOCK,
                _ => 0,
            };

            Ok(flags as u64)
        },
        fs::F_SETFL => {
            if let FdKind::Pipe(pipe) = fd_kind(fd)? {
                pipe.set_nonblock(argument & O_NONBLOCK as u64 != 0);
            }

            Ok(0)
        },
        _ => Err(KError::EINVAL),
    }
}

pub fn umask(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let _mask = frame.rdi as u32;
    Ok(0)
}
