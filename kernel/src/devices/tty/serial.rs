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

use spin::Once;

use crate::arch::{Arch, Processor};

static SERIAL: Once<SerialPort> = Once::new();

const COM1: u16 = 0x3F8;

pub struct SerialPort {
    base: u16,
}

impl SerialPort {
    fn new(base: u16) -> Self {
        let serial = Self { base };
        serial.install();
        serial
    }

    fn install(&self) {
        // straight up copied from:
        // https://wiki.osdev.org/Serial_Ports

        self.write(1, 0x00); // Disable all interrupts
        self.write(3, 0x80); // Enable DLAB (set baud rate divisor)
        self.write(0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        self.write(1, 0x00); //                  (hi byte)
        self.write(3, 0x03); // 8 bits, no parity, one stop bit
        self.write(2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        self.write(4, 0x0B); // IRQs enabled, RTS/DSR set
        self.write(4, 0x1E); // Set in loopback mode, test the serial chip
        self.write(0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i.e: not same byte as sent)
        if self.read(0) != 0xAE {
            panic!("serial port not functioning properly");
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        self.write(4, 0x0F);
    }

    fn write(&self, offset: u16, value: u8) {
        Processor::write::<u8>(self.base as u32 + offset as u32, value)
    }

    fn read(&self, offset: u16) -> u8 {
        Processor::read::<u8>(self.base as u32 + offset as u32)
    }

    fn send(&self, byte: u8) {
        for _ in 0..100_000 {
            if self.read(5) & 0x20 != 0 {
                self.write(0, byte);
                return;
            }

            core::hint::spin_loop();
        }
    }
}

impl fmt::Write for &SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send(b'\r');
            }
            self.send(byte);
        }

        Ok(())
    }
}

pub fn install() {
    SERIAL.call_once(|| {
        let serial = SerialPort::new(COM1);
        serial.install();
        serial
    });
}

pub fn current() -> &'static SerialPort {
    SERIAL.get().expect("serial port not installed")
}

pub fn print(args: fmt::Arguments<'_>) {
    Processor::interrupts_without(|| {
        let mut serial = self::current();
        serial.write_fmt(args).expect("failed to write to serial port");
    });
}

pub fn ready() -> bool { SERIAL.is_completed() }
