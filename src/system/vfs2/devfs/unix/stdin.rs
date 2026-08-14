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

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::ToString;

use spin::Mutex;

use crate::devices::ps2::keyboard::{KEYBOARD, KeyEvent, KeyboardSubscriber};
use crate::system::vfs2::devfs::{DevFile, DevFs};
use crate::system::vfs2::error::VfsResult;

static KB_BUFFER: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());

struct DevFSKeyboard;

impl KeyboardSubscriber for DevFSKeyboard {
    fn on_key_event(&mut self, event: KeyEvent) {
        if let KeyEvent::Ascii(byte) = event {
            KB_BUFFER.lock().push_back(byte);
        }
    }
}

fn kb_read(_offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
    let mut queue = KB_BUFFER.lock();
    let mut read = 0;

    for out in buf.iter_mut() {
        let Some(byte) = queue.pop_front() else {
            break;
        };

        *out = byte;
        read += 1;
    }

    Ok(read)
}

fn kb_write(_buf: &[u8]) -> VfsResult<usize> { Ok(0) }

pub fn install(dev: &mut DevFs) {
    let subscriber = Box::leak(Box::new(DevFSKeyboard));
    KEYBOARD.lock().subscribe(subscriber);

    dev.bind(DevFile::new(
        "/keyboard".to_string(),
        Some(kb_read),
        Some(kb_write),
        None,
    ));
}
