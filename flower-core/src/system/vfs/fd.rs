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

use crate::system;
use crate::system::vfs::error::{VfsError, VfsResult};
use crate::system::vfs::file::{File, OpenFlags};
use crate::system::vfs::perm::Credentials;

pub const DEFAULT_MAX_FDS: usize = 128;

pub struct FdTable {
    fds: Vec<Option<Arc<File>>>,
    limit: usize,
}

impl FdTable {
    pub fn new(limit: usize) -> Self {
        let mut fds: Vec<Option<Arc<File>>> =
            Vec::with_capacity(limit.min(DEFAULT_MAX_FDS));

        fds.push(Some(
            system::vfs::open(
                "/dev/stdin",
                OpenFlags::RDONLY,
                Credentials::ROOT,
            )
            .expect("failed to open /dev/stdin"),
        ));

        fds.push(Some(
            system::vfs::open(
                "/dev/stdout",
                OpenFlags::WRONLY,
                Credentials::ROOT,
            )
            .expect("failed to open /dev/stdout"),
        ));

        fds.push(Some(
            system::vfs::open(
                "/dev/stderr",
                OpenFlags::WRONLY,
                Credentials::ROOT,
            )
            .expect("failed to open /dev/stderr"),
        ));

        Self { fds, limit: limit.min(DEFAULT_MAX_FDS) }
    }

    pub fn install(&mut self, file: Arc<File>) -> VfsResult<usize> {
        if let Some(idx) = self.fds.iter().position(|slot| slot.is_none()) {
            self.fds[idx] = Some(file);
            return Ok(idx);
        }

        if self.fds.len() >= self.limit {
            return Err(VfsError::NoSpace);
        }

        self.fds.push(Some(file));
        Ok(self.fds.len() - 1)
    }

    pub fn install_at(&mut self, fd: usize, file: Arc<File>) -> VfsResult<()> {
        if fd >= self.limit {
            return Err(VfsError::NoSpace);
        }

        if fd >= self.fds.len() {
            self.fds.resize(fd + 1, None);
        }

        self.fds[fd] = Some(file);
        Ok(())
    }

    pub fn get(&self, fd: usize) -> VfsResult<Arc<File>> {
        self.fds.get(fd).and_then(|slot| slot.clone()).ok_or(VfsError::NotFound)
    }

    pub fn dup(&mut self, fd: usize) -> VfsResult<usize> {
        let file = self.get(fd)?;
        self.install(file)
    }

    pub fn dup2(&mut self, old_fd: usize, new_fd: usize) -> VfsResult<()> {
        if old_fd == new_fd {
            return self.get(old_fd).map(|_| ());
        }

        let file = self.get(old_fd)?;
        self.install_at(new_fd, file)
    }

    pub fn close(&mut self, fd: usize) -> VfsResult<()> {
        let slot = self.fds.get_mut(fd).ok_or(VfsError::NotFound)?;
        if slot.is_none() {
            return Err(VfsError::NotFound);
        }
        *slot = None;
        Ok(())
    }
}

impl Clone for FdTable {
    fn clone(&self) -> Self {
        Self { fds: self.fds.clone(), limit: self.limit }
    }
}
