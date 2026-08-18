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

use crate::devices::ps2::mouse::handler::MOUSE_SUB;
use crate::devices::ps2::mouse::publisher::{
    MouseButton, MouseEvent, MouseSubscriber,
};
use crate::system::vfs::devfs::{DevFile, DevFs};
use crate::system::vfs::error::VfsResult;

const BTN_LEFT_BIT: u8 = 0x01;
const BTN_RIGHT_BIT: u8 = 0x02;
const BTN_MIDDLE_BIT: u8 = 0x04;
const ALWAYS_ONE_BIT: u8 = 0x08;
const X_SIGN_BIT: u8 = 0x10;
const Y_SIGN_BIT: u8 = 0x20;

static MOUSE_BUFFER: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());

struct DevFSMouse {
    left: bool,
    right: bool,
    middle: bool,
}

impl DevFSMouse {
    const fn new() -> Self { Self { left: false, right: false, middle: false } }

    fn flush_packet(&self, dx: i16, dy: i16) {
        let mut flags = ALWAYS_ONE_BIT;
        if self.left {
            flags |= BTN_LEFT_BIT;
        }
        if self.right {
            flags |= BTN_RIGHT_BIT;
        }
        if self.middle {
            flags |= BTN_MIDDLE_BIT;
        }

        let dx_clamped = dx.clamp(-256, 255);
        let dy_clamped = dy.clamp(-256, 255);

        if dx_clamped < 0 {
            flags |= X_SIGN_BIT;
        }
        if dy_clamped < 0 {
            flags |= Y_SIGN_BIT;
        }

        let mut queue = MOUSE_BUFFER.lock();
        queue.push_back(flags);
        queue.push_back(dx_clamped as u8);
        queue.push_back(dy_clamped as u8);
    }
}

impl MouseSubscriber for DevFSMouse {
    fn on_mouse_event(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::Move { dx, dy } => {
                self.flush_packet(dx, dy);
            },
            MouseEvent::ButtonPress(button) => {
                match button {
                    MouseButton::Left => self.left = true,
                    MouseButton::Right => self.right = true,
                    MouseButton::Middle => self.middle = true,
                }
                self.flush_packet(0, 0);
            },
            MouseEvent::ButtonRelease(button) => {
                match button {
                    MouseButton::Left => self.left = false,
                    MouseButton::Right => self.right = false,
                    MouseButton::Middle => self.middle = false,
                }
                self.flush_packet(0, 0);
            },
        }
    }
}

fn mouse_read(_offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
    let mut queue = MOUSE_BUFFER.lock();
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

fn mouse_write(_buf: &[u8]) -> VfsResult<usize> { Ok(0) }

pub(crate) fn bind(dev: &mut DevFs) {
    let subscriber = Box::leak(Box::new(DevFSMouse::new()));
    MOUSE_SUB.lock().subscribe(subscriber);

    dev.bind(DevFile::new(
        "/mouse0".to_string(),
        Some(mouse_read),
        Some(mouse_write),
        None,
        None,
    ));
}
