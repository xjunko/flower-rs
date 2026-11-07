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

use spin::Mutex;

use crate::arch::addr::KPhysAddr;
use crate::boot::limine::FRAMEBUFFER_REQUEST;

#[derive(Clone, Copy, Debug)]
pub struct Framebuffer {
    pub addr: KPhysAddr,

    pub width: u64,
    pub height: u64,

    pub pitch: u64,
    pub bpp: u16,
}

impl Framebuffer {
    pub fn draw(&self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let offset = y * self.pitch as usize + x * self.bpp as usize / 8;

        unsafe {
            let pixel = self.addr.as_ptr::<u8>().add(offset) as *mut u32;
            *pixel =
                (rgb.0 as u32) << 16 | (rgb.1 as u32) << 8 | (rgb.2 as u32);
        }
    }
}

static FRAMEBUFFER: Mutex<Option<Framebuffer>> = Mutex::new(None);

pub fn install() {
    if let Some(framebuffer) = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("no framebuffers")
        .framebuffers()
        .next()
    {
        let fb = Framebuffer {
            addr: KPhysAddr::new(framebuffer.addr() as u64),
            width: framebuffer.width(),
            height: framebuffer.height(),
            pitch: framebuffer.pitch(),
            bpp: framebuffer.bpp(),
        };

        *FRAMEBUFFER.lock() = Some(fb);
    }
}

pub fn ready() -> bool { FRAMEBUFFER.lock().is_some() }

pub fn get() -> Option<Framebuffer> { *FRAMEBUFFER.lock() }
