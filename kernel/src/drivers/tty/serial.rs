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

use core::fmt::{self, Write};

use spin::Mutex;

use super::CrlfWriter;
use crate::arch;

struct SerialPort {
    base: u16,
}

impl SerialPort {
    // honestly this seems a little bit hacky
    // because if the status line is stuck it's most likely
    // that something terribly has gone wrong, but whatever
    // limit the wait anyway....
    const MAX_WAIT: usize = 100_000;

    const fn new(base: u16) -> Self { Self { base } }

    fn init(&mut self) {
        self.write(1, 0x00);
        self.write(3, 0x80);
        self.write(0, 0x03);
        self.write(1, 0x00);
        self.write(3, 0x03);
        self.write(2, 0xC7);
        self.write(4, 0x0B);
    }

    fn write(&self, offset: u16, value: u8) {
        arch::ports::write::<u8>(self.base as u32 + offset as u32, value)
    }

    fn read(&self, offset: u16) -> u8 {
        arch::ports::read::<u8>(self.base as u32 + offset as u32)
    }

    fn send(&self, byte: u8) {
        for _ in 0..Self::MAX_WAIT {
            if self.read(5) & 0x20 != 0 {
                self.write(0, byte);
                return;
            }

            core::hint::spin_loop();
        }
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.send(c as u8);
        }
        Ok(())
    }
}

const _DEFAULT_COM_PORT: u16 = 0x3F8;
static SERIAL: Mutex<SerialPort> =
    Mutex::new(SerialPort::new(_DEFAULT_COM_PORT));

pub fn install() { SERIAL.lock().init() }

pub fn write_fmt(args: fmt::Arguments) {
    let mut serial = SERIAL.lock();
    let mut writer = CrlfWriter::new(&mut *serial);
    let _ = writer.write_fmt(args);
}

pub fn try_write_fmt(args: fmt::Arguments) {
    if let Some(mut serial) = SERIAL.try_lock() {
        let mut writer = CrlfWriter::new(&mut *serial);
        let _ = writer.write_fmt(args);
    }
}
