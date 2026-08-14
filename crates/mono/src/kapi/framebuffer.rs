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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct fb_info {
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    pub pitch: u32,
}

impl fb_info {
    pub const SIZE: usize = core::mem::size_of::<fb_info>();

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];

        buf[0..4].copy_from_slice(&self.width.to_le_bytes());
        buf[4..8].copy_from_slice(&self.height.to_le_bytes());
        buf[8..12].copy_from_slice(&self.bpp.to_le_bytes());
        buf[12..16].copy_from_slice(&self.pitch.to_le_bytes());

        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::SIZE {
            return None;
        }

        Some(Self {
            width: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            height: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            bpp: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            pitch: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
        })
    }
}
