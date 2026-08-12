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

use spin::Once;

use crate::boot::limine::FRAMEBUFFER_REQUEST;

static FB0: Once<LimineFramebuffer> = Once::new();

unsafe impl Send for LimineFramebuffer {}
unsafe impl Sync for LimineFramebuffer {}

pub struct LimineFramebuffer {
    buffer: *mut u8,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
    pub pitch: usize,
}

impl LimineFramebuffer {
    fn new() -> Option<LimineFramebuffer> {
        if let Some(framebuffer) = FRAMEBUFFER_REQUEST
            .get_response()
            .expect("no framebuffer")
            .framebuffers()
            .next()
        {
            let fb = LimineFramebuffer {
                buffer: framebuffer.addr(),
                width: framebuffer.width() as usize,
                height: framebuffer.height() as usize,
                bpp: framebuffer.bpp() as usize,
                pitch: framebuffer.pitch() as usize,
            };

            log::debug!(
                "Framebuffer: Addr={:#x} Res={}x{} Col={}bpp",
                fb.buffer as usize,
                fb.width,
                fb.height,
                fb.bpp
            );

            return Some(fb);
        }

        None
    }

    pub fn addr(&self) -> *mut u8 { self.buffer }

    pub fn size(&self) -> (usize, usize) { (self.width, self.height) }

    pub fn draw_pixel(&self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let offset = y * self.pitch + x * self.bpp / 8;
        unsafe {
            let pixel = self.buffer.add(offset) as *mut u32;
            *pixel =
                (rgb.0 as u32) << 16 | (rgb.1 as u32) << 8 | (rgb.2 as u32);
        }
    }
}

pub fn get() -> Option<&'static LimineFramebuffer> { FB0.get() }

pub fn install() {
    if let Some(fb) = LimineFramebuffer::new() {
        FB0.call_once(|| fb);
    } else {
        panic!("no framebuffer found");
    }
}
