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
use spinning_top::Spinlock;
use uart_16550::backend::PioBackend;
use uart_16550::{Config, Uart16550Tty};

type ComPort = Uart16550Tty<PioBackend>;
static SERIAL: Once<SerialPort> = Once::new();

static COM1: u16 = 0x3F8;

pub struct SerialPort {
    uart: Spinlock<ComPort>,
}

impl SerialPort {
    fn new() -> Self {
        let uart = unsafe {
            ComPort::new_port(COM1, Config::default()).expect("invalid COM1")
        };

        Self { uart: Spinlock::new(uart) }
    }

    pub fn write_formatted(&self, args: fmt::Arguments<'_>) {
        let _ = self.uart.lock().write_fmt(args);
    }
}

pub fn install() { SERIAL.call_once(SerialPort::new); }

pub fn current() -> &'static SerialPort {
    SERIAL.get().expect("serial port not installed")
}

pub fn print(args: fmt::Arguments<'_>) { current().write_formatted(args); }

pub fn ready() -> bool { SERIAL.is_completed() }
