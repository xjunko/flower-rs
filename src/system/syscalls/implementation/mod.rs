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

use flower_mono::syscalls::{
    SYS_ARCHCTL, SYS_CLOSE, SYS_EXECVE, SYS_EXIT, SYS_FORK, SYS_MMAP,
    SYS_MPROTECT, SYS_MSLEEP, SYS_MTIME, SYS_MUNMAP, SYS_OPEN, SYS_READ,
    SYS_SEEK, SYS_STAT, SYS_WAITPID, SYS_WRITE,
};

mod arch;
mod fs;
mod mman;
mod process;

use crate::system::syscalls::types::SyscallHandler;

pub static SYSCALL_HANDLERS: [Option<SyscallHandler>; 256] = {
    let mut handlers = [None; 256];

    handlers[SYS_EXIT as usize] = Some(process::exit as SyscallHandler);
    handlers[SYS_FORK as usize] = Some(process::fork as SyscallHandler);
    handlers[SYS_WAITPID as usize] = Some(process::waitpid as SyscallHandler);
    handlers[SYS_EXECVE as usize] = Some(process::execve as SyscallHandler);

    handlers[SYS_READ as usize] = Some(fs::read as SyscallHandler);
    handlers[SYS_WRITE as usize] = Some(fs::write as SyscallHandler);
    handlers[SYS_OPEN as usize] = Some(fs::open as SyscallHandler);
    handlers[SYS_CLOSE as usize] = Some(fs::close as SyscallHandler);
    handlers[SYS_SEEK as usize] = Some(fs::seek as SyscallHandler);
    handlers[SYS_STAT as usize] = Some(fs::stat as SyscallHandler);

    handlers[SYS_MSLEEP as usize] = Some(process::msleep as SyscallHandler);
    handlers[SYS_MTIME as usize] = Some(process::mtime as SyscallHandler);

    handlers[SYS_ARCHCTL as usize] = Some(arch::ctl as SyscallHandler);

    handlers[SYS_MMAP as usize] = Some(mman::mmap as SyscallHandler);
    handlers[SYS_MUNMAP as usize] = Some(mman::munmap as SyscallHandler);
    handlers[SYS_MPROTECT as usize] = Some(mman::mprotect as SyscallHandler);

    handlers
};
