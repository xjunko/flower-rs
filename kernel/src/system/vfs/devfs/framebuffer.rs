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

use alloc::string::ToString;
use core::ffi::c_int;

use crate::boot::limine::FRAMEBUFFER_REQUEST;
use crate::system::vfs::devfs::{DevFS, DevFile};
use crate::system::vfs::{VFSError, VFSResult};

fn framebuffer_read(_offset: usize, _buf: &mut [u8]) -> usize {
    unimplemented!()
}
fn framebuffer_write(_buf: &[u8]) -> usize { unimplemented!() }

fn framebuffer_mmap(
    _size: usize,
    _prot: c_int,
    _flags: c_int,
    _offset: u64,
) -> VFSResult<*mut u8> {
    if let Some(fb) = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("no valid framebuffer")
        .framebuffers()
        .next()
    {
        // should we map a virtual address
        Ok(fb.addr())
    } else {
        Err(VFSError::NotFound)
    }
}

pub fn install(dev: &mut DevFS) {
    dev.bind(DevFile::new(
        "/fb0".to_string(),
        Some(framebuffer_read),
        Some(framebuffer_write),
        Some(framebuffer_mmap),
    ));
}
