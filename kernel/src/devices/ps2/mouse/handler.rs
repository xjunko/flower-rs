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
use crate::devices::ps2::mouse::publisher::{
    MouseButton, MouseEvent, MousePublisher, MouseState,
};
use crate::devices::ps2::mouse::{PS2_DATA, PS2_STATUS};

pub static MOUSE_SUB: LazyLock<Mutex<MousePublisher>> =
    LazyLock::new(|| Mutex::new(MousePublisher::new()));

static PACKET_INDEX: Mutex<usize> = Mutex::new(0);
static PACKET: Mutex<[u8; 3]> = Mutex::new([0; 3]);
static PREV_BUTTONS: LazyLock<Mutex<(bool, bool, bool)>> =
    LazyLock::new(|| Mutex::new((false, false, false)));

fn parse_packet(bytes: [u8; 3]) -> Option<MouseState> {
    let flags = bytes[0];

    // overflows, just drop it.
    if flags & 0xC0 != 0 {
        return None;
    }

    let mut dx = bytes[1] as i16;
    let mut dy = bytes[2] as i16;

    if flags & 0x10 != 0 {
        dx -= 256;
    }
    if flags & 0x20 != 0 {
        dy -= 256;
    }

    Some(MouseState {
        dx,
        dy,
        left: flags & 0x01 != 0,
        right: flags & 0x02 != 0,
        middle: flags & 0x04 != 0,
    })
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(
    _frame: InterruptStackFrame,
) {
    let mut status_port: Port<u8> = Port::new(PS2_STATUS);
    let mut data_port: Port<u8> = Port::new(PS2_DATA);

    // check if mouse data is available
    let status = unsafe { status_port.read() };
    if status & 0x01 == 0 || status & 0x20 == 0 {
        apic::eoi();
        return;
    }

    let byte = unsafe { data_port.read() };

    let mut index = PACKET_INDEX.lock();
    let mut packet = PACKET.lock();

    // probably invalid packet
    if *index == 0 && byte & 0x08 == 0 {
        drop(packet);
        drop(index);
        apic::eoi();
        return;
    }

    packet[*index] = byte;
    *index += 1;

    if *index == 3 {
        if let Some(state) = parse_packet(*packet) {
            if state.dx != 0 || state.dy != 0 {
                MOUSE_SUB
                    .lock()
                    .publish(MouseEvent::Move { dx: state.dx, dy: state.dy });
            }

            let mut prev = PREV_BUTTONS.lock();
            publish_button_diff(prev.0, state.left, MouseButton::Left);
            publish_button_diff(prev.1, state.right, MouseButton::Right);
            publish_button_diff(prev.2, state.middle, MouseButton::Middle);
            *prev = (state.left, state.right, state.middle);
        }
        *index = 0;
    }

    drop(packet);
    drop(index);

    apic::eoi();
}

fn publish_button_diff(was_down: bool, is_down: bool, button: MouseButton) {
    if is_down && !was_down {
        MOUSE_SUB.lock().publish(MouseEvent::ButtonPress(button));
    } else if !is_down && was_down {
        MOUSE_SUB.lock().publish(MouseEvent::ButtonRelease(button));
    }
}
