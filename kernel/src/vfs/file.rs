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

use spin::Mutex;

use crate::vfs::traits::{File, FileOps, Inode};
use crate::vfs::{Metadata, SeekFrom, VfsError, VfsResult};

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct OpenFlags: u32 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
        const APPEND = 0b00000100;
        const CREATE = 0b00001000;
        const TRUNCATE = 0b00010000;
    }
}

pub struct OpenFile {
    inode: Arc<dyn Inode>,
    ops: Arc<dyn FileOps>,
    offset: Mutex<u64>,
    flags: OpenFlags,
}

impl OpenFile {
    pub fn new(
        inode: Arc<dyn Inode>,
        ops: Arc<dyn FileOps>,
        flags: OpenFlags,
    ) -> Self {
        Self { inode, ops, offset: Mutex::new(0), flags }
    }

    pub fn close(&self) -> VfsResult<()> { self.inode.close() }

    pub fn inode(&self) -> Arc<dyn Inode> { self.inode.clone() }

    pub fn ops(&self) -> Arc<dyn FileOps> { self.ops.clone() }

    pub fn flags(&self) -> OpenFlags { self.flags }
}

impl Drop for OpenFile {
    fn drop(&mut self) { let _ = self.close(); }
}

impl File for OpenFile {
    fn read(&self, buf: &mut [u8]) -> VfsResult<usize> {
        if !self.flags.contains(OpenFlags::READ) {
            return Err(VfsError::PermissionDenied);
        }

        let mut offset = self.offset.lock();
        let read = self.ops.read(*offset, buf)?;
        *offset += read as u64;
        Ok(read as usize)
    }

    fn write(&self, buf: &[u8]) -> VfsResult<usize> {
        if !self.flags.contains(OpenFlags::WRITE) {
            return Err(VfsError::PermissionDenied);
        }

        let mut offset = self.offset.lock();
        if self.flags.contains(OpenFlags::APPEND) {
            *offset = self.inode.metadata()?.size;
        }
        let written = self.ops.write(*offset, buf)?;
        *offset += written as u64;
        Ok(written as usize)
    }

    fn seek(&self, at: SeekFrom) -> VfsResult<()> {
        let mut offset = self.offset.lock();

        let new_offset = match at {
            SeekFrom::Start(off) => off as i64,
            SeekFrom::End(off) => self.inode.metadata()?.size as i64 + off,
            SeekFrom::Current(off) => *offset as i64 + off,
        };

        if new_offset < 0 {
            return Err(VfsError::InvalidArgument);
        }

        *offset = new_offset as u64;
        Ok(())
    }

    fn metadata(&self) -> VfsResult<Metadata> { self.inode.metadata() }
}
