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

use x86_64::instructions::port::Port;

mod defs;
pub mod handler;
pub mod publisher;

const KB_DEVICE: u16 = 0x60;
const KB_PENDING: u16 = 0x64;

const MAX_DRAIN: usize = 32;

pub fn install() {
    let mut pending_port: Port<u8> = Port::new(KB_PENDING);
    let mut data_port: Port<u8> = Port::new(KB_DEVICE);

    // optimally this should get all the
    // pending scancodes cleared out.
    for _ in 0..MAX_DRAIN {
        if unsafe { pending_port.read() } & 0x1 == 0 {
            break;
        }
        let _ = unsafe { data_port.read() };
    }
    log::debug!("ps2::keyboard installed!");
}
