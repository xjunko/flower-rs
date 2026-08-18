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

use spin::{LazyLock, Mutex};
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

use crate::arch::x86_64::apic;
use crate::devices::ps2::keyboard::defs::{
    scancode_to_ascii, scancode_to_keycode,
};
use crate::devices::ps2::keyboard::publisher::{KeyEvent, KeyboardPublisher};
use crate::devices::ps2::keyboard::{KB_DEVICE, KB_PENDING};

pub static KEYBOARD_SUB: LazyLock<Mutex<KeyboardPublisher>> =
    LazyLock::new(|| Mutex::new(KeyboardPublisher::new()));

static SHIFT_PRESSED: LazyLock<Mutex<bool>> =
    LazyLock::new(|| Mutex::new(false));

pub extern "x86-interrupt" fn keyboard_interrupt_handler(
    _frame: InterruptStackFrame,
) {
    let mut pending_port: Port<u8> = Port::new(KB_PENDING);
    let mut data_port: Port<u8> = Port::new(KB_DEVICE);

    let pending = unsafe { pending_port.read() };
    if pending & 0x1 == 0 {
        apic::eoi();
        return;
    }

    let scancode = unsafe { data_port.read() };

    let mut shift_pressed = SHIFT_PRESSED.lock();
    if scancode == 0x2A || scancode == 0x36 {
        *shift_pressed = true;
    } else if scancode == 0xAA || scancode == 0xB6 {
        *shift_pressed = false;
    }

    let keycode = scancode_to_keycode(scancode & 0x7F);
    let event = if scancode & 0x80 == 0 {
        KeyEvent::Press(keycode as u8)
    } else {
        KeyEvent::Release(keycode as u8)
    };
    KEYBOARD_SUB.lock().publish(event);

    if scancode & 0x80 == 0
        && let Some(ascii) = scancode_to_ascii(scancode, *shift_pressed)
    {
        KEYBOARD_SUB.lock().publish(KeyEvent::Ascii(ascii));
    }

    apic::eoi();
}
