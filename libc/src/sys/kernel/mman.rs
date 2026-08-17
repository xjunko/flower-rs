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

use flower_mono::mmap::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use flower_mono::syscalls::{SYS_MMAP, SYS_MUNMAP};

use crate::sys::kernel::{syscall_result, syscall2, syscall6};

pub fn mmap(
    addr: *mut u8,
    size: usize,
    prot: u64,
    flags: u64,
    fd: u64,
    offset: usize,
) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    let ret = syscall6(
        SYS_MMAP,
        addr as u64,
        size as u64,
        prot,
        flags,
        fd,
        offset as u64,
    );
    let ret = syscall_result(ret);

    if ret < 0 { core::ptr::null_mut() } else { ret as *mut u8 }
}

pub fn mmap_anonymous(size: usize) -> *mut u8 {
    mmap(
        core::ptr::null_mut(),
        size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        u64::MAX,
        0,
    )
}

pub fn munmap(addr: *mut u8, size: usize) -> i64 {
    if addr.is_null() || size == 0 {
        return -1;
    }

    let ret = syscall_result(syscall2(SYS_MUNMAP, addr as u64, size as u64));
    if ret < 0 { -1 } else { 0 }
}
