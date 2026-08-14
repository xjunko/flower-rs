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

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ffi::c_int;

use spin::Mutex;

use crate::system::vfs::error::{VfsError, VfsResult};
use crate::system::vfs::inode::{DirectoryEntry, FileType, Inode, Metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenFlags(u32);

impl OpenFlags {
    pub const APPEND: Self = Self(1 << 4);
    pub const CREATE: Self = Self(1 << 2);
    pub const DIRECTORY: Self = Self(1 << 5);
    pub const EXCLUSIVE: Self = Self(1 << 6);
    pub const RDONLY: Self = Self(0);
    pub const RDWR: Self = Self(1 << 1);
    pub const TRUNCATE: Self = Self(1 << 3);
    pub const WRONLY: Self = Self(1 << 0);

    pub const fn from_bits(bits: u32) -> Self { Self(bits) }

    pub const fn bits(&self) -> u32 { self.0 }

    pub fn contains(&self, flag: Self) -> bool { self.0 & flag.0 != 0 }

    pub fn readable(&self) -> bool {
        !self.contains(Self::WRONLY) || self.contains(Self::RDWR)
    }

    pub fn writable(&self) -> bool {
        self.contains(Self::WRONLY) || self.contains(Self::RDWR)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Whence {
    Start,
    Current,
    End,
}

pub struct File {
    inode: Arc<dyn Inode>,
    offset: Mutex<usize>,
    flags: OpenFlags,
}

impl File {
    pub fn new(inode: Arc<dyn Inode>, flags: OpenFlags) -> Self {
        Self { inode, offset: Mutex::new(0), flags }
    }

    pub fn inode(&self) -> &Arc<dyn Inode> { &self.inode }

    pub fn flags(&self) -> OpenFlags { self.flags }

    pub fn read(&self, buf: &mut [u8]) -> VfsResult<usize> {
        if !self.flags.readable() {
            return Err(VfsError::PermissionDenied);
        }

        let mut offset = self.offset.lock();
        let read = self.inode.read_at(*offset, buf)?;
        *offset += read;
        Ok(read)
    }

    pub fn write(&self, buf: &[u8]) -> VfsResult<usize> {
        if !self.flags.writable() {
            return Err(VfsError::PermissionDenied);
        }

        let mut offset = self.offset.lock();
        if self.flags.contains(OpenFlags::APPEND) {
            *offset += self.inode.metadata()?.size;
        }

        let written = self.inode.write_at(*offset, buf)?;
        *offset += written;
        Ok(written)
    }

    pub fn seek(&self, delta: i64, whence: Whence) -> VfsResult<usize> {
        let mut offset = self.offset.lock();

        let base = match whence {
            Whence::Start => 0i64,
            Whence::Current => *offset as i64,
            Whence::End => self.inode.metadata()?.size as i64,
        };

        let new_offset =
            base.checked_add(delta).ok_or(VfsError::InvalidArgument)?;

        if new_offset < 0 {
            return Err(VfsError::InvalidArgument);
        }

        *offset = new_offset as usize;
        Ok(*offset)
    }

    pub fn metadata(&self) -> VfsResult<Metadata> { self.inode.metadata() }

    pub fn mmap(
        &self,
        len: usize,
        prot: c_int,
        flags: c_int,
        offset: u64,
    ) -> VfsResult<*mut u8> {
        self.inode.mmap(len, prot, flags, offset)
    }

    pub fn readdir(&self) -> VfsResult<Vec<DirectoryEntry>> {
        if self.inode.file_type()? != FileType::Directory {
            return Err(VfsError::NotADirectory);
        }

        self.inode.readdir()
    }
}
