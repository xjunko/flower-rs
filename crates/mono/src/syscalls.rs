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

pub const SYS_RESTART: u64 = 0;
pub const SYS_EXIT: u64 = 1;
pub const SYS_FORK: u64 = 2;
pub const SYS_READ: u64 = 3;
pub const SYS_WRITE: u64 = 4;
pub const SYS_OPEN: u64 = 5;
pub const SYS_CLOSE: u64 = 6;
pub const SYS_WAITPID: u64 = 7;
pub const SYS_SEEK: u64 = 8;
pub const SYS_EXECVE: u64 = 9;
pub const SYS_STAT: u64 = 10;

pub const SYS_ARCHCTL: u64 = 157;

pub const SYS_MMAP: u64 = 31;
pub const SYS_MUNMAP: u64 = 32;
pub const SYS_MPROTECT: u64 = 33;

pub const SYS_MSLEEP: u64 = 101;
pub const SYS_MTIME: u64 = 102;

// old riria-specific impl
pub const SYS_WRITE_FS_BASE: u64 = 29;
pub const SYS_GET_THREAD_ID: u64 = 30;
