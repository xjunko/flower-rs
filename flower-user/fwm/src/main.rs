#![no_std]
#![no_main]
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

extern crate alloc;
mod graphic;

use alloc::string::ToString;
use flower_libc::{file::File, print, sys::fs::bits::FS_RDWR, thread::sleep};

use crate::graphic::{Graphic, color::Color};

const FB_WIDTH: usize = 1280;
const FB_HEIGHT: usize = 720;
const FB_PITCH: usize = FB_WIDTH * 4;
const FB_BPP: usize = 32;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let fb = match File::open("/dev/fb0".to_string(), FS_RDWR) {
        Ok(f) => f,
        Err(_) => {
            flower_libc::println!("failed to open /dev/fb0");
            return -1;
        }
    };

    let fb_addr = match fb.mmap(FB_PITCH * FB_HEIGHT) {
        Ok(addr) => addr,
        Err(_) => {
            flower_libc::println!("failed to mmap /dev/fb0");
            return -1;
        }
    };

    let mut gg = Graphic::new(fb_addr, FB_WIDTH, FB_HEIGHT, FB_PITCH, FB_BPP);

    loop {
        gg.draw_rect(0, 0, FB_WIDTH, FB_HEIGHT, Color::new(0, 0, 0, 255));
        gg.draw_rect(0, 0, FB_WIDTH, 16, Color::new(255, 255, 255, 255));
        gg.flush();
    }
}
