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

use flower_mono::numbers::{
    SYS_ACCESS, SYS_ARCH_PRCTL, SYS_BRK, SYS_CLOCK_GETTIME, SYS_CLOSE,
    SYS_EXECVE, SYS_EXIT, SYS_EXIT_GROUP, SYS_FORK, SYS_GET_TID, SYS_IOCTL,
    SYS_MMAP, SYS_MPROTECT, SYS_MSLEEP, SYS_MTIME, SYS_MUNMAP, SYS_NANOSLEEP,
    SYS_OPEN, SYS_PIPE, SYS_READ, SYS_READV, SYS_SEEK, SYS_SET_TID_ADDRESS,
    SYS_SIG_PROC_MASK, SYS_STAT, SYS_TIMES, SYS_UMASK, SYS_UNAME, SYS_WAIT4,
    SYS_WAITPID, SYS_WRITE, SYS_WRITEV,
};

mod arch;
mod fs;
mod memory;
mod process;
mod signal;
mod system;
mod thread;
mod time;

use crate::system::syscalls::types::SyscallHandler;

macro_rules! syscall_handlers {
    ($($number:ident => $handler:path),* $(,)?) => {{
        let mut handlers: [Option<SyscallHandler>; 256] = [None; 256];

        $(
            handlers[$number as usize] = Some($handler as SyscallHandler);
        )*

        handlers
    }};
}

pub static SYSCALL_HANDLERS: [Option<SyscallHandler>; 256] = syscall_handlers! {
    SYS_EXIT            => process::exit,
    SYS_EXIT_GROUP      => process::exit_group,
    SYS_FORK            => process::fork,
    SYS_WAITPID         => process::waitpid,
    SYS_WAIT4           => process::wait4,
    SYS_EXECVE          => process::execve,

    SYS_GET_TID         => thread::get_tid,
    SYS_SET_TID_ADDRESS => thread::set_tid_address,

    SYS_MSLEEP          => time::msleep,
    SYS_NANOSLEEP       => time::nanosleep,
    SYS_MTIME           => time::mtime,
    SYS_CLOCK_GETTIME   => time::clock_get_time,
    SYS_TIMES           => time::times,

    SYS_SIG_PROC_MASK   => signal::signal_process_mask,

    SYS_READ        => fs::read,
    SYS_READV       => fs::read_v,
    SYS_WRITE       => fs::write,
    SYS_WRITEV      => fs::write_v,
    SYS_OPEN        => fs::open,
    SYS_CLOSE       => fs::close,
    SYS_SEEK        => fs::seek,
    SYS_STAT        => fs::stat,
    SYS_IOCTL       => fs::ioctl,
    SYS_ACCESS      => fs::access,
    SYS_PIPE        => fs::pipe,
    SYS_UMASK       => fs::umask,

    SYS_ARCH_PRCTL  => arch::prctl,

    SYS_UNAME       => system::uname,

    SYS_MMAP        => memory::mmap,
    SYS_MUNMAP      => memory::munmap,
    SYS_MPROTECT    => memory::mprotect,
    SYS_BRK         => memory::brk,
};
