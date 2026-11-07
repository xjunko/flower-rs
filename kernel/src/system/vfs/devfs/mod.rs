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

mod audio;
mod framebuffer;
mod keyboard;
mod proc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::c_int;
use core::sync::atomic::{AtomicUsize, Ordering};

pub use proc::create_procfs;

use crate::arch::addr::KPhysAddr;
use crate::system::posix::errno::{KError, KResult};
use crate::system::vfs::{File, FileLike, FileSystem, FileType, Stat, Whence};

type ReadFn = fn(usize, &mut [u8]) -> usize;
type WriteFn = fn(&[u8]) -> usize;
type MmapFn = fn(usize, c_int, c_int, u64) -> KResult<KPhysAddr>;

pub struct DevFile {
    path: String,
    position: AtomicUsize,

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
            path,
            position: AtomicUsize::new(0),
            fn_read: read,
            fn_write: write,
            fn_mmap: mmap,
        }
    }
}

impl Clone for DevFile {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            position: AtomicUsize::new(0),
            fn_read: self.fn_read,
            fn_write: self.fn_write,
            fn_mmap: self.fn_mmap,
        }
    }
}

impl File for DevFile {
    fn name(&self) -> KResult<String> { Ok(self.path.clone()) }

    fn read(&self, buf: &mut [u8]) -> KResult<usize> {
        if let Some(read_fn) = self.fn_read {
            let position = self.position.load(Ordering::Acquire);
            let read = read_fn(position, buf);
            if read > 0 {
                self.position.fetch_add(read, Ordering::AcqRel);
            }
            Ok(read)
        } else {
            Err(KError::EOPNOTSUPP)
        }
    }

    fn write(&self, buf: &mut [u8]) -> KResult<usize> {
        if let Some(write_fn) = self.fn_write {
            Ok(write_fn(buf))
        } else {
            Err(KError::EOPNOTSUPP)
        }
    }

    fn seek(&mut self, offset: i64, pos: Whence) -> KResult<usize> {
        let current = self.position.load(Ordering::Acquire);
        let new_pos = match pos {
            Whence::Start => offset as usize,
            Whence::Current => current.saturating_add(offset as usize),
            Whence::End => offset as usize, // HACK: just treat it as is
        };

        self.position.store(new_pos, Ordering::Release);
        Ok(new_pos)
    }

    fn mmap(
        &self,
        len: usize,
        prot: core::ffi::c_int,
        flags: core::ffi::c_int,
        offset: u64,
    ) -> KResult<KPhysAddr> {
        if let Some(mmap_fn) = self.fn_mmap {
            mmap_fn(len, prot, flags, offset)
        } else {
            Err(KError::EOPNOTSUPP)
        }
    }

    fn stat(&self) -> KResult<Stat> {
        Ok(Stat {
            name: self
                .path
                .split("/")
                .last()
                .unwrap_or(self.path.as_str())
                .to_string(),
            typ: FileType::Device,
            size: 0,
            last_modified: 0,
        })
    }
}

pub struct DevFS {
    files: Vec<DevFile>,
}

impl DevFS {
    pub fn new() -> Self { Self { files: Vec::new() } }

    pub fn bind(&mut self, file: DevFile) { self.files.push(file); }
}

impl FileSystem for DevFS {
    // DevFS is special since it can easily be used on the other parts of the kernel, so we don't need to initialize it
    fn initialize(&mut self) -> KResult<()> { Ok(()) }

    fn open(&self, path: &str, _flags: u32) -> KResult<FileLike> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .map(|f| FileLike::File(Box::new(f.clone())))
            .ok_or(KError::EEXIST)
    }

    fn stat(&self, path: &str) -> KResult<Stat> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.stat())
            .ok_or(KError::EEXIST)?
    }
}

pub fn create_devfs() -> DevFS {
    let mut mnt = DevFS::new();
    keyboard::install(&mut mnt);
    audio::install(&mut mnt);
    framebuffer::install(&mut mnt);
    mnt
}
