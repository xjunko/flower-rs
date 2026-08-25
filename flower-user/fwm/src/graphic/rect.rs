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

use crate::graphic::{Graphic, color::Color};

impl Graphic {
    #[inline]
    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = y * self.pitch + x * (self.bpp / 8);

        unsafe {
            self.back
                .as_mut_ptr()
                .add(offset)
                .cast::<u32>()
                .write_unaligned(color.to_u32());
        }
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: Color) {
        let Some(end_x) = x.checked_add(width) else {
            return;
        };
        let Some(end_y) = y.checked_add(height) else {
            return;
        };

        let start_x = x.min(self.width);
        let start_y = y.min(self.height);
        let end_x = end_x.min(self.width);
        let end_y = end_y.min(self.height);

        if start_x >= end_x || start_y >= end_y {
            return;
        }

        for y in start_y..end_y {
            for x in start_x..end_x {
                self.put_pixel(x, y, color);
            }
        }
    }

    pub fn draw_rect_empty(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: Color,
    ) {
        let Some(end_x) = x.checked_add(width) else {
            return;
        };
        let Some(end_y) = y.checked_add(height) else {
            return;
        };

        let start_x = x.min(self.width);
        let start_y = y.min(self.height);
        let end_x = end_x.min(self.width);
        let end_y = end_y.min(self.height);

        if start_x >= end_x || start_y >= end_y {
            return;
        }

        for x in start_x..end_x {
            self.put_pixel(x, start_y, color);

            if end_y > start_y + 1 {
                self.put_pixel(x, end_y - 1, color);
            }
        }

        for y in (start_y + 1)..(end_y - 1) {
            self.put_pixel(start_x, y, color);

            if end_x > start_x + 1 {
                self.put_pixel(end_x - 1, y, color);
            }
        }
    }

    pub fn draw_line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, color: Color) {
        let mut x = x1 as isize;
        let mut y = y1 as isize;

        let target_x = x2 as isize;
        let target_y = y2 as isize;

        let dx = (target_x - x).abs();
        let dy = (target_y - y).abs();

        let sx = if x < target_x { 1 } else { -1 };
        let sy = if y < target_y { 1 } else { -1 };

        let mut err = dx - dy;

        loop {
            if x >= 0 && y >= 0 {
                self.put_pixel(x as usize, y as usize, color);
            }

            if x == target_x && y == target_y {
                break;
            }

            let e2 = 2 * err;

            if e2 > -dy {
                err -= dy;
                x += sx;
            }

            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
}
