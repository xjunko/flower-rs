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

use crate::vfs::file::OpenFlags;
use crate::vfs::{FileType, Metadata, SeekFrom, VfsError, VfsResult};

/// generic filesystem interface
pub trait FileSystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn root(&self) -> VfsResult<Arc<dyn Inode>>;
    fn metadata(&self) -> VfsResult<Metadata>;
    fn sync(&self) -> VfsResult<()>;
}

/// generic inode interface
pub trait Inode: Send + Sync {
    fn open(&self, flags: OpenFlags) -> VfsResult<Arc<dyn FileOps>>;

    fn close(&self) -> VfsResult<()>;

    fn metadata(&self) -> VfsResult<Metadata> { Err(VfsError::InvalidArgument) }

    fn get_type(&self) -> VfsResult<FileType> { Err(VfsError::InvalidArgument) }

    fn lookup(&self, _name: &str) -> VfsResult<Arc<dyn Inode>> {
        Err(VfsError::NotADirectory)
    }

    fn create(
        &self,
        _name: &str,
        _flags: OpenFlags,
    ) -> VfsResult<Arc<dyn Inode>> {
        Err(VfsError::NotADirectory)
    }

    fn readlink(&self) -> VfsResult<String> { Err(VfsError::InvalidArgument) }
}

/// vfs-backed file ops
pub trait FileOps: Send + Sync {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize>;
    fn write(&self, offset: u64, buf: &[u8]) -> VfsResult<usize>;
    fn truncate(&self, size: u64) -> VfsResult<()>;
}

/// generic file interface
pub trait File {
    fn read(&self, buf: &mut [u8]) -> VfsResult<usize>;
    fn write(&self, buf: &[u8]) -> VfsResult<usize>;
    fn seek(&self, at: SeekFrom) -> VfsResult<()>;
    fn metadata(&self) -> VfsResult<Metadata>;
}
