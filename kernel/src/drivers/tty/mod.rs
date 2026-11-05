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

use alloc::fmt;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};

use x86_64::instructions::interrupts;

pub mod serial;
pub mod terminal;

static CONSOLE_BUSY: AtomicBool = AtomicBool::new(false);

struct ConsoleGuard;

impl ConsoleGuard {
    fn acquire() -> Option<Self> {
        CONSOLE_BUSY
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| Self)
    }
}

impl Drop for ConsoleGuard {
    fn drop(&mut self) { CONSOLE_BUSY.store(false, Ordering::Release); }
}

static LOGGING: AtomicBool = AtomicBool::new(false);

struct LogGuard;

impl LogGuard {
    fn acquire() -> Option<Self> {
        LOGGING
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| Self)
    }
}

impl Drop for LogGuard {
    fn drop(&mut self) { LOGGING.store(false, Ordering::Release); }
}

struct CrlfWriter<'a, W: Write> {
    inner: &'a mut W,
}

impl<'a, W: Write> CrlfWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self { Self { inner } }
}

impl<W: Write> Write for CrlfWriter<'_, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut start = 0;

        for (index, byte) in s.bytes().enumerate() {
            if byte == b'\n' {
                if start < index {
                    self.inner.write_str(&s[start..index])?;
                }
                self.inner.write_str("\r\n")?;
                start = index + 1;
            }
        }

        if start < s.len() {
            self.inner.write_str(&s[start..])?;
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::drivers::tty::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    if CONSOLE_BUSY.load(Ordering::Acquire) || LOGGING.load(Ordering::Acquire) {
        interrupts::without_interrupts(|| serial::try_write_fmt(args));
        return;
    }

    let Some(_guard) = ConsoleGuard::acquire() else {
        interrupts::without_interrupts(|| serial::try_write_fmt(args));
        return;
    };

    interrupts::without_interrupts(|| {
        serial::write_fmt(args);
        terminal::write_fmt(args);
    });
}

pub fn _log(args: fmt::Arguments) {
    if CONSOLE_BUSY.load(Ordering::Acquire) {
        return;
    }

    let Some(_guard) = LogGuard::acquire() else {
        return;
    };

    interrupts::without_interrupts(|| {
        serial::try_write_fmt(args);
        terminal::try_write_fmt(args);
    });
}
