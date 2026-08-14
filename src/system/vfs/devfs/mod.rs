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

pub mod proc;
pub mod unix;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ffi::c_int;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::system::vfs::error::{VfsError, VfsResult};
use crate::system::vfs::fs::FileSystem;
use crate::system::vfs::inode::{DirectoryEntry, FileType, Inode, Metadata};
use crate::system::vfs::perm::Permissions;

type ReadFn = fn(usize, &mut [u8]) -> VfsResult<usize>;
type WriteFn = fn(&[u8]) -> VfsResult<usize>;
type MmapFn = fn(usize, c_int, c_int, u64) -> VfsResult<*mut u8>;

static NEXT_INODE: AtomicU64 = AtomicU64::new(1);

fn next_inode() -> u64 { NEXT_INODE.fetch_add(1, Ordering::SeqCst) }

pub struct DevFile {
    inode: u64,
    path: String,

    fn_read: Option<ReadFn>,
    fn_write: Option<WriteFn>,
    fn_mmap: Option<MmapFn>,
}

impl DevFile {
    pub fn new(
        path: String,
        read: Option<ReadFn>,
        write: Option<WriteFn>,
        mmap: Option<MmapFn>,
    ) -> Self {
        Self {
            inode: next_inode(),
            path,
            fn_read: read,
            fn_write: write,
            fn_mmap: mmap,
        }
    }

    fn name(&self) -> &str { self.path.trim_start_matches("/") }
}

impl Inode for DevFile {
    fn metadata(&self) -> VfsResult<Metadata> {
        Ok(Metadata {
            inode: self.inode,
            size: 0,
            file_type: FileType::CharDevice,
            permissions: Permissions::from_unix(0o666),
            owner: 0,
            group: 0,
            links: 1,
            last_modified: 0,
        })
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
        match self.fn_read {
            Some(read_fn) => read_fn(offset, buf),
            None => Err(VfsError::Unsupported),
        }
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) -> VfsResult<usize> {
        match self.fn_write {
            Some(write_fn) => write_fn(buf),
            None => Err(VfsError::Unsupported),
        }
    }

    fn mmap(
        &self,
        len: usize,
        prot: c_int,
        flags: c_int,
        offset: u64,
    ) -> VfsResult<*mut u8> {
        match self.fn_mmap {
            Some(mmap_fn) => mmap_fn(len, prot, flags, offset),
            None => Err(VfsError::Unsupported),
        }
    }
}

struct DevDirectory {
    inode: u64,
    entries: Vec<(String, Arc<dyn Inode>)>,
}
impl Inode for DevDirectory {
    fn metadata(&self) -> VfsResult<Metadata> {
        Ok(Metadata {
            inode: self.inode,
            size: 0,
            file_type: FileType::Directory,
            permissions: Permissions::from_unix(0o755),
            owner: 0,
            group: 0,
            links: 2,
            last_modified: 0,
        })
    }

    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn Inode>> {
        self.entries
            .iter()
            .find(|(entry_name, _)| entry_name == name)
            .map(|(_, inode)| inode.clone())
            .ok_or(VfsError::NotFound)
    }

    fn readdir(&self) -> VfsResult<Vec<super::inode::DirectoryEntry>> {
        self.entries
            .iter()
            .map(|(name, inode)| {
                let meta = inode.metadata()?;
                Ok(DirectoryEntry {
                    name: name.clone(),
                    inode: meta.inode,
                    file_type: meta.file_type,
                })
            })
            .collect()
    }
}

pub struct DevFs {
    root_inode: u64,
    entries: Vec<(String, Arc<dyn Inode>)>,
}

impl DevFs {
    pub fn new() -> Self {
        Self { root_inode: next_inode(), entries: Vec::new() }
    }

    pub fn bind(&mut self, file: DevFile) {
        let name = file.name().to_string();
        self.entries.push((name, Arc::new(file)));
    }
}

impl FileSystem for DevFs {
    fn name(&self) -> &str { "devfs" }

    fn root(&self) -> Arc<dyn Inode> {
        Arc::new(DevDirectory {
            inode: self.root_inode,
            entries: self.entries.clone(),
        })
    }

    fn sync(&self) -> VfsResult<()> { todo!() }
}
