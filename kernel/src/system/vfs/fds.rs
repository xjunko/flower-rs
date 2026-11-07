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

use crate::system::mem::vma::Shmem;
use crate::system::posix::errno::{KError, KResult};
use crate::system::vfs::pipe::KPipeEnd;
use crate::system::vfs::types::File;

pub const MAX_FDS: usize = 256;

/// an entry in a process' fd table.
#[derive(Clone)]
pub enum FdKind {
    File(Arc<dyn File>),
    Pipe(Arc<KPipeEnd>),
    Shmem(Arc<Shmem>),
    Stdin,
    Stdout,
    Stderr,
}

pub struct FdTable {
    fds: Vec<Option<FdKind>>,
}

impl FdTable {
    pub fn new() -> Self {
        let mut table = Self { fds: alloc::vec![None; 3] };
        table.fds[0] = Some(FdKind::Stdin);
        table.fds[1] = Some(FdKind::Stdout);
        table.fds[2] = Some(FdKind::Stderr);
        table
    }

    pub fn alloc(&mut self, kind: FdKind) -> KResult<usize> {
        for (fd, slot) in self.fds.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(kind);
                return Ok(fd);
            }
        }

        if self.fds.len() >= MAX_FDS {
            return Err(KError::EMFILE);
        }

        self.fds.push(Some(kind));
        Ok(self.fds.len() - 1)
    }

    pub fn get(&self, fd: usize) -> KResult<&FdKind> {
        self.fds.get(fd).and_then(|slot| slot.as_ref()).ok_or(KError::EBADF)
    }

    pub fn get_cloned(&self, fd: usize) -> KResult<FdKind> {
        self.get(fd).cloned()
    }

    pub fn set(&mut self, fd: usize, kind: FdKind) -> KResult<()> {
        if fd >= MAX_FDS {
            return Err(KError::EBADF);
        }

        if fd >= self.fds.len() {
            self.fds.resize(fd + 1, None);
        }

        self.fds[fd] = Some(kind);
        Ok(())
    }

    pub fn close(&mut self, fd: usize) -> KResult<()> {
        let slot = self.fds.get_mut(fd).ok_or(KError::EBADF)?;

        if slot.take().is_none() {
            return Err(KError::EBADF);
        }

        Ok(())
    }
}

impl Default for FdTable {
    fn default() -> Self { Self::new() }
}

impl Clone for FdTable {
    fn clone(&self) -> Self { Self { fds: self.fds.clone() } }
}
