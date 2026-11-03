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

use x86_64::instructions::interrupts;

pub mod logging;
pub mod serial;
pub mod terminal;

struct CrlfWriter<'a, W: Write> {
    inner: &'a mut W,
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
    interrupts::without_interrupts(|| {
        let mut serial = super::tty::serial::SERIAL.lock();
        let _ = serial.write_fmt(args);

        let mut guard = terminal::get();
        if let Some(term) = guard.as_mut() {
            let mut writer = CrlfWriter { inner: term };
            let _ = writer.write_fmt(args);
        }
    });
}
