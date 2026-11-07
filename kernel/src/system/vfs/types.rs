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
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ffi::c_int;

use crate::arch::addr::KPhysAddr;
use crate::system::mem::vma::Shmem;
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

#[derive(Clone)]
pub enum KFile {
    File(Arc<dyn File>),
    Directory(Arc<dyn Directory>),
    Shmem(Arc<Shmem>),
}

pub enum KMapBacking {
    Physical(KPhysAddr),
    Shmem(Arc<Shmem>),
}

pub trait File: Send + Sync {
    fn name(&self) -> KResult<String> { Err(KError::ENOSYS) }

    fn stat(&self) -> KResult<Stat> { Err(KError::ENOSYS) }

    fn read(&self, buf: &mut [u8]) -> KResult<usize>;

    fn write(&self, buf: &[u8]) -> KResult<usize>;

    fn seek(&self, offset: i64, pos: Whence) -> KResult<usize>;

    fn ioctl(&self, _cmd: u64, _arg: usize) -> KResult<usize> {
        Err(KError::ENOSYS)
    }

    fn mmap(
        &self,
        _len: usize,
        _prot: c_int,
        _flags: c_int,
        _offset: u64,
    ) -> KResult<KMapBacking> {
        Err(KError::ENOSYS)
    }
}

pub trait Directory: Send + Sync {
    fn name(&self) -> KResult<String>;

    fn contents(&self) -> KResult<Vec<KFile>>;

    fn delete(&self, name: &str) -> KResult<()>;
}

pub trait FileSystem: Send + Sync {
    fn initialize(&mut self) -> KResult<()>;

    fn open(&self, path: &str, flags: u32) -> KResult<KFile>;

    fn unlink(&self, _path: &str) -> KResult<()> { Err(KError::EROFS) }

    fn stat(&self, path: &str) -> KResult<Stat>;

    fn exists(&self, path: &str) -> bool { self.stat(path).is_ok() }
}
