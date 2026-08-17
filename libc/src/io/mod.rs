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

use crate::print;
use crate::sys::fs::{self, open};

pub fn getch() -> u8 {
    let path = b"/dev/stdin\0";
    let kb = open(path.as_ptr(), path.len(), 0, 0);
    if kb < 0 {
        return 0;
    }

    let mut c = [0u8; 1];
    loop {
        let _ = fs::read(kb as u64, c.as_mut_ptr(), 1);
        if c[0] != 0 {
            fs::close(kb as u64);
            return c[0];
        }
    }
}

pub fn read_line(buf: &mut [u8]) -> usize {
    let mut pos = 0;
    loop {
        let c = getch();

        match c {
            b'\n' => {
                print!("\n");
                return pos;
            },
            b'\x08' => {
                if pos > 0 {
                    pos -= 1;
                    print!("\x08 \x08");
                }
            },
            32..126 => {
                if pos < buf.len() {
                    buf[pos] = c;
                    pos += 1;
                    print!("{}", c as char);
                }
            },
            _ => break,
        }
    }
    0
}
