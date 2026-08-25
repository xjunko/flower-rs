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

use alloc::vec;
use flower_libc::thread::sleep;

pub mod color;
mod rect;

pub struct Graphic {
    addr: *mut u8,
    back: vec::Vec<u8>,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: usize,
}

impl Graphic {
    pub fn new(addr: *mut u8, width: usize, height: usize, pitch: usize, bpp: usize) -> Self {
        let back = vec![0; pitch * height];
        Self {
            addr,
            back,
            width,
            height,
            pitch,
            bpp,
        }
    }

    pub fn flush(&mut self) {
        unsafe {
            core::ptr::copy_nonoverlapping(self.back.as_ptr(), self.addr, self.back.len());
        }
    }
}
