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

use alloc::boxed::Box;

use crate::system::posix::errno::{KError, KResult};
use crate::system::vfs::types::File;

pub const MAX_FDS: usize = 8;

pub enum FdKind {
    File(Box<dyn File>),
    Stdin,
    Stdout,
    Stderr,
}

pub struct FdTable {
    fds: [Option<FdKind>; MAX_FDS],
}

impl FdTable {
    pub fn new() -> Self {
        let mut table = Self { fds: core::array::from_fn(|_| None) };
        table.fds[0] = Some(FdKind::Stdin);
        table.fds[1] = Some(FdKind::Stdout);
        table.fds[2] = Some(FdKind::Stderr);
        table
    }

    pub fn alloc(&mut self, kind: FdKind) -> KResult<usize> {
        for (i, slot) in self.fds.iter_mut().enumerate().skip(3) {
            if slot.is_none() {
                *slot = Some(kind);
                return Ok(i);
            }
        }
        Err(KError::ENOSPC)
    }

    pub fn get(&self, fd: usize) -> KResult<&FdKind> {
        if fd >= MAX_FDS {
            return Err(KError::EBADFD);
        }
        self.fds.get(fd).and_then(|opt| opt.as_ref()).ok_or(KError::EBADFD)
    }

    pub fn get_mut(&mut self, fd: usize) -> KResult<&mut FdKind> {
        if fd >= MAX_FDS {
            return Err(KError::EBADFD);
        }
        self.fds.get_mut(fd).and_then(|opt| opt.as_mut()).ok_or(KError::EBADF)
    }

    pub fn close(&mut self, fd: usize) -> KResult<()> {
        if fd >= MAX_FDS {
            return Err(KError::EBADFD);
        }
        if fd < 3 {
            return Err(KError::EPERM);
        }
        if self.fds[fd].is_none() {
            return Err(KError::EBADFD);
        }
        self.fds[fd] = None;
        Ok(())
    }
}

impl Clone for FdTable {
    fn clone(&self) -> Self {
        let mut table = Self::new();
        for (fd, slot) in self.fds.iter().enumerate().skip(3) {
            match slot.as_ref() {
                Some(FdKind::Stdin) => table.fds[fd] = Some(FdKind::Stdin),
                Some(FdKind::Stdout) => table.fds[fd] = Some(FdKind::Stdout),
                Some(FdKind::Stderr) => table.fds[fd] = Some(FdKind::Stderr),
                Some(FdKind::File(_)) | None => {},
            }
        }
        table
    }
}
