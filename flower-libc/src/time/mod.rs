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

use flower_uapi::syscalls::SYS_MTIME;

use crate::sys::kernel::syscall1;

pub struct Duration {
    nanos: u64,
}

impl Duration {
    pub fn as_nanos(&self) -> u64 { self.nanos }

    pub fn as_millis(&self) -> u64 { self.nanos / 1_000_000 }

    pub fn as_secs(&self) -> u64 { self.nanos / 1_000_000_000 }
}

pub struct SystemTime {
    nanos: u64,
}

impl SystemTime {
    pub fn now() -> Self { SystemTime { nanos: __sys_get_time_ns() } }

    pub fn elapsed(&self) -> Duration {
        Duration { nanos: self.elapsed_nanos() }
    }

    pub fn elapsed_nanos(&self) -> u64 { __sys_get_time_ns() - self.nanos }

    pub fn as_nanos(&self) -> u64 { self.nanos }

    pub fn as_millis(&self) -> u64 { self.nanos / 1_000_000 }

    pub fn as_secs(&self) -> u64 { self.nanos / 1_000_000_000 }
}

fn __sys_get_time_ns() -> u64 {
    let mut time: u64 = 0;
    syscall1(SYS_MTIME, &mut time as *mut u64 as u64);
    time
}
