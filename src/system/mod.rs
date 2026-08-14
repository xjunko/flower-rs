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

use crate::system::syscalls::SyscallError;
use crate::system::vfs2::error::VfsError;

pub mod elf;
pub mod proc;
pub mod smp;
pub mod syscalls;
pub mod vfs2;

pub enum KernelError {
    FileSystem(VfsError),
}

pub type KernelResult<T> = Result<T, KernelError>;

pub trait ToSyscallError {
    fn to_syscall_error(&self) -> SyscallError;
}

impl ToSyscallError for KernelError {
    fn to_syscall_error(&self) -> SyscallError {
        match self {
            Self::FileSystem(err) => err.to_syscall_error(),
        }
    }
}
