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

use flower_mono::syscalls::SYS_MTIME;

use crate::sys::kernel::syscall1;

pub struct Duration {
    millis: u64,
}

impl Duration {
    pub fn as_millis(&self) -> u64 { self.millis }

    pub fn as_secs(&self) -> u64 { self.millis / 1000 }
}

pub struct SystemTime {
    millis: u64,
}

impl SystemTime {
    pub fn now() -> Self { SystemTime { millis: __sys_get_time_ms() } }

    pub fn elapsed(&self) -> Duration {
        Duration { millis: self.elapsed_millis() }
    }

    pub fn elapsed_millis(&self) -> u64 { __sys_get_time_ms() - self.millis }

    pub fn as_millis(&self) -> u64 { self.millis }
}

fn __sys_get_time_ms() -> u64 {
    let mut time: u64 = 0;
    syscall1(SYS_MTIME, &mut time as *mut u64 as u64);
    time
}
