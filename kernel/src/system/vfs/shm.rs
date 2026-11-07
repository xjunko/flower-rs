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
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use flower_abi::fs::{O_CREAT, O_EXCL};
use spin::Mutex;

use crate::system::mem::vma::Shmem;
use crate::system::posix::errno::{KError, KResult};
use crate::system::vfs::{FileSystem, FileType, KFile, Stat};

pub struct SharedMemoryFS {
    objects: Mutex<BTreeMap<String, Arc<Shmem>>>,
}

impl SharedMemoryFS {
    pub fn new() -> Self { Self { objects: Mutex::new(BTreeMap::new()) } }
}

impl Default for SharedMemoryFS {
    fn default() -> Self { Self::new() }
}

impl FileSystem for SharedMemoryFS {
    fn initialize(&mut self) -> KResult<()> { Ok(()) }

    fn open(&self, path: &str, flags: u32) -> KResult<KFile> {
        let mut objects = self.objects.lock();

        if let Some(shmem) = objects.get(path) {
            if flags & (O_CREAT | O_EXCL) == O_CREAT | O_EXCL {
                return Err(KError::EEXIST);
            }

            return Ok(KFile::Shmem(Arc::clone(shmem)));
        }

        if flags & O_CREAT == 0 {
            return Err(KError::ENOENT);
        }

        let shmem = Shmem::new(0);
        objects.insert(path.to_string(), Arc::clone(&shmem));

        Ok(KFile::Shmem(shmem))
    }

    fn unlink(&self, path: &str) -> KResult<()> {
        self.objects.lock().remove(path).map(|_| ()).ok_or(KError::ENOENT)
    }

    fn stat(&self, path: &str) -> KResult<Stat> {
        let objects = self.objects.lock();
        let shmem = objects.get(path).ok_or(KError::ENOENT)?;

        Ok(Stat {
            name: path.trim_start_matches('/').to_string(),
            size: shmem.len(),
            typ: FileType::File,
            last_modified: 0,
        })
    }
}
