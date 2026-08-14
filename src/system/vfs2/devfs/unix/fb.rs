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

use flower_mono::kapi::framebuffer::fb_info;

use crate::devices;
use crate::system::vfs2::devfs::{DevFile, DevFs};
use crate::system::vfs2::error::{VfsError, VfsResult};

fn fb_mmap(
    _size: usize,
    _prot: c_int,
    _flags: c_int,
    _offset: u64,
) -> VfsResult<*mut u8> {
    if let Some(fb) = devices::gpu::fb::get() {
        // NOTE: limine already maps the framebuffer.
        Ok(fb.addr().as_mut_ptr::<u8>())
    } else {
        Err(VfsError::NotFound)
    }
}

fn fb_info_read(_offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
    let info = match devices::gpu::fb::get() {
        Some(fb) => fb_info {
            width: fb.width as u32,
            height: fb.height as u32,
            bpp: fb.bpp as u32,
            pitch: fb.pitch as u32,
        },
        None => return Err(VfsError::NotFound),
    };

    if buf.len() < fb_info::SIZE {
        return Err(VfsError::InvalidArgument);
    }

    buf[..fb_info::SIZE].copy_from_slice(info.to_bytes().as_slice());

    Ok(fb_info::SIZE)
}

pub(crate) fn bind(dev: &mut DevFs) {
    dev.bind(DevFile::new(
        "/fb0".to_string(),
        Some(fb_info_read),
        None,
        Some(fb_mmap),
    ));
}
