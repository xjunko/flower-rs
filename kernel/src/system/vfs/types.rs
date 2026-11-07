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
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_int;

use crate::arch::addr::KPhysAddr;
use crate::system::posix::errno::{KError, KResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    File,
    Directory,
    Device,
    Symlink,
    Pipe,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Stat {
    pub name: String,
    pub size: usize,
    pub typ: FileType,
    pub last_modified: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Whence {
    Start,
    Current,
    End,
}

pub enum FileLike {
    File(Box<dyn File>),
    Directory(Box<dyn Directory>),
}

pub trait File: Send + Sync {
    /// gets the name of the file
    fn name(&self) -> KResult<String> { Err(KError::ENOSYS) }

    /// gets the info for the file
    fn stat(&self) -> KResult<Stat> { Err(KError::ENOSYS) }

    /// reads data into the given buffer and returns the number of bytes read
    fn read(&self, buf: &mut [u8]) -> KResult<usize>;

    /// writes data from the given buffer and returns the number of bytes written
    fn write(&self, buf: &mut [u8]) -> KResult<usize>;

    /// seeks to the given position and returns the new position
    fn seek(&mut self, offset: i64, pos: Whence) -> KResult<usize>;

    /// ioctl
    fn ioctl(&self, _cmd: u64, _arg: usize) -> KResult<usize> {
        Err(KError::ENOSYS)
    }

    /// maps the file into memory and returns a pointer to the mapped region
    fn mmap(
        &self,
        _len: usize,
        _prot: c_int,
        _flags: c_int,
        _offset: u64,
    ) -> KResult<KPhysAddr> {
        Err(KError::ENOSYS)
    }
}

pub trait Directory: Send + Sync {
    /// get the name of the directory
    fn name(&self) -> KResult<String>;

    /// get the contents of the directory
    fn contents(&self) -> KResult<Vec<FileLike>>;

    /// deletes a file in the directory
    fn delete(&self, name: &str) -> KResult<()>;
}

pub trait FileSystem: Send + Sync {
    /// initializes the filesystem
    fn initialize(&mut self) -> KResult<()>;

    /// opens the file
    fn open(&self, path: &str, flags: u32) -> KResult<FileLike>;

    /// gets the info for the file
    fn stat(&self, path: &str) -> KResult<Stat>;

    /// checks if the file exists
    fn exists(&self, path: &str) -> bool { self.stat(path).is_ok() }
}
