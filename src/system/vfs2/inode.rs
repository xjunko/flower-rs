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

use crate::system::vfs2::error::VfsResult;
use crate::system::vfs2::perm::Permissions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Regular,
    Directory,
    CharDevice,
    BlockDevice,
    Symlink,
    Fifo,
    Socket,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub inode: u64,
    pub size: usize,
    pub file_type: FileType,
    pub permissions: Permissions,
    pub owner: u32,
    pub group: u32,
    pub links: usize,
    pub last_modified: u64,
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub inode: u64,
    pub file_type: FileType,
}

pub trait Inode: Send + Sync {
    // info
    fn metadata(&self) -> VfsResult<Metadata>;

    fn file_type(&self) -> VfsResult<FileType> {
        Ok(self.metadata()?.file_type)
    }

    // file ops
    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> VfsResult<usize> {
        todo!()
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> VfsResult<usize> {
        todo!()
    }

    fn truncate(&self, _size: usize) -> VfsResult<usize> { todo!() }

    fn mmap(
        &self,
        _len: usize,
        _prot: c_int,
        _flags: c_int,
        _offset: u64,
    ) -> VfsResult<*mut u8> {
        todo!()
    }

    // dir ops
    fn lookup(&self, _name: &str) -> VfsResult<Arc<dyn Inode>> { todo!() }

    fn readdir(&self) -> VfsResult<Vec<DirectoryEntry>> { todo!() }

    fn create(
        &self,
        _name: &str,
        _file_type: FileType,
    ) -> VfsResult<Arc<dyn Inode>> {
        todo!()
    }

    fn unlink(&self, _name: &str) -> VfsResult<()> { todo!() }

    fn rename(
        &self,
        _old: &str,
        _new_parent: &dyn Inode,
        _new: &str,
    ) -> VfsResult<()> {
        todo!()
    }

    // symlink ops
    fn readlink(&self) -> VfsResult<String> { todo!() }

    // perms ps
    fn chmod(&self, _mode: u16) -> VfsResult<()> { todo!() }

    fn chown(&self, _uid: u32, _gid: u32) -> VfsResult<()> { todo!() }
}
