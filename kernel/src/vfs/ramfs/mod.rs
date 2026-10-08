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

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::Mutex;

use crate::vfs::file::OpenFlags;
use crate::vfs::traits::{FileOps, FileSystem, Inode};
use crate::vfs::{FileType, Metadata, VfsError, VfsResult};

#[derive(Clone, Copy, PartialEq, Eq)]
enum RamInodeType {
    File,
    Directory,
}

pub struct RamFs {
    root: Arc<RamInode>,
}

impl RamFs {
    pub fn new() -> Self {
        let ids = Arc::new(AtomicU64::new(2));
        Self {
            root: Arc::new(RamInode {
                inode: 1,
                kind: RamInodeType::Directory,
                ids,
                data: Arc::new(Mutex::new(Vec::new())),
                children: Arc::new(Mutex::new(BTreeMap::new())),
            }),
        }
    }
}

impl Default for RamFs {
    fn default() -> Self { Self::new() }
}

impl FileSystem for RamFs {
    fn name(&self) -> &'static str { "ramfs" }

    fn root(&self) -> VfsResult<Arc<dyn Inode>> { Ok(self.root.clone()) }

    fn metadata(&self) -> VfsResult<Metadata> { self.root.metadata() }

    fn sync(&self) -> VfsResult<()> { Ok(()) }
}

struct RamInode {
    inode: u64,
    kind: RamInodeType,
    ids: Arc<AtomicU64>,
    data: Arc<Mutex<Vec<u8>>>,
    children: Arc<Mutex<BTreeMap<String, Arc<RamInode>>>>,
}

impl Inode for RamInode {
    fn open(&self, _flags: OpenFlags) -> VfsResult<Arc<dyn FileOps>> {
        if self.kind != RamInodeType::File {
            return Err(VfsError::NotAFile);
        }

        Ok(Arc::new(RamFileOps { data: self.data.clone() }))
    }

    fn close(&self) -> VfsResult<()> { Ok(()) }

    fn metadata(&self) -> VfsResult<Metadata> {
        let size = match self.kind {
            RamInodeType::File => self.data.lock().len() as u64,
            RamInodeType::Directory => self.children.lock().len() as u64,
        };
        Ok(Metadata { inode: self.inode, size })
    }

    fn get_type(&self) -> VfsResult<FileType> {
        Ok(match self.kind {
            RamInodeType::File => FileType::File,
            RamInodeType::Directory => FileType::Directory,
        })
    }

    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn Inode>> {
        if self.kind != RamInodeType::Directory {
            return Err(VfsError::NotADirectory);
        }

        self.children
            .lock()
            .get(name)
            .cloned()
            .map(|inode| inode as Arc<dyn Inode>)
            .ok_or(VfsError::NotFound)
    }

    fn create(
        &self,
        name: &str,
        _flags: OpenFlags,
    ) -> VfsResult<Arc<dyn Inode>> {
        if self.kind != RamInodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        if name.is_empty() || name.contains('/') {
            return Err(VfsError::InvalidPath);
        }

        let mut children = self.children.lock();
        if children.contains_key(name) {
            return Err(VfsError::AlreadyExists);
        }

        let inode = Arc::new(RamInode {
            inode: self.ids.fetch_add(1, Ordering::Relaxed),
            kind: RamInodeType::File,
            ids: self.ids.clone(),
            data: Arc::new(Mutex::new(Vec::new())),
            children: Arc::new(Mutex::new(BTreeMap::new())),
        });
        children.insert(String::from(name), inode.clone());
        Ok(inode as Arc<dyn Inode>)
    }
}

struct RamFileOps {
    data: Arc<Mutex<Vec<u8>>>,
}

impl FileOps for RamFileOps {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let data = self.data.lock();
        let start =
            usize::try_from(offset).map_err(|_| VfsError::InvalidArgument)?;
        if start >= data.len() {
            return Ok(0);
        }

        let count = buf.len().min(data.len() - start);
        buf[..count].copy_from_slice(&data[start..start + count]);
        Ok(count)
    }

    fn write(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut data = self.data.lock();
        let start =
            usize::try_from(offset).map_err(|_| VfsError::InvalidArgument)?;
        let end =
            start.checked_add(buf.len()).ok_or(VfsError::InvalidArgument)?;
        if end > data.len() {
            data.resize(end, 0);
        }
        data[start..end].copy_from_slice(buf);
        Ok(buf.len())
    }

    fn truncate(&self, size: u64) -> VfsResult<()> {
        let size =
            usize::try_from(size).map_err(|_| VfsError::InvalidArgument)?;
        self.data.lock().resize(size, 0);
        Ok(())
    }
}
