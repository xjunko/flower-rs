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

use flower_mono::kapi::framebuffer::{fb_draw_pixel, fb_info};

use crate::devices;
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
    if let Some(fb) = devices::gpu::fb::get() {
        // NOTE: limine already maps the framebuffer.
        Ok(fb.addr().as_mut_ptr::<u8>())
    } else {
        Err(VFSError::NotFound)
    }
}

fn framebuffer_info(_offset: usize, buf: &mut [u8]) -> usize {
    let info = match devices::gpu::fb::get() {
        Some(fb) => fb_info {
            width: fb.width as u32,
            height: fb.height as u32,
            bpp: fb.bpp as u32,
            pitch: fb.pitch as u32,
        },
        None => return 0,
    };

    if buf.len() < fb_info::SIZE {
        return 0;
    }

    buf[..fb_info::SIZE].copy_from_slice(info.to_bytes().as_slice());

    fb_info::SIZE
}

fn framebuffer_draw_pixel(buf: &[u8]) -> usize {
    if buf.len() < fb_draw_pixel::SIZE {
        return 0;
    }

    if let Some(draw) = fb_draw_pixel::from_bytes(buf)
        && let Some(fb) = devices::gpu::fb::get()
    {
        fb.draw_pixel(
            draw.x as usize,
            draw.y as usize,
            (draw.r, draw.g, draw.b),
        );
    }

    0
}

pub fn install(dev: &mut DevFS) {
    dev.bind(DevFile::new(
        "/fb0".to_string(),
        Some(framebuffer_read),
        Some(framebuffer_write),
        Some(framebuffer_mmap),
    ));

    dev.bind(DevFile::new(
        "/fb0/info".to_string(),
        Some(framebuffer_info),
        None,
        None,
    ));

    dev.bind(DevFile::new(
        "/fb0/draw".to_string(),
        None,
        Some(framebuffer_draw_pixel),
        None,
    ));
}
