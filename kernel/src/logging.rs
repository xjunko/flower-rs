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

use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

use crate::arch::{Arch, Processor};
use crate::devices;

static TTY: SerialTty = SerialTty;
const TTY_LEVEL: LevelFilter = LevelFilter::Info;

struct SerialTty;

impl log::Log for SerialTty {
    fn enabled(&self, _metadata: &log::Metadata) -> bool { true }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let content = record.args();
            let time_secs = Processor::timer_get_ns() as f64 / 1_000_000_000.0;

            match record.level() {
                Level::Error => {
                    __print(format_args!(
                        "[{:>7.3}] {}: {}:{} {} \n",
                        time_secs,
                        "error".bright_red(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.red().bold()
                    ));
                },
                Level::Warn => {
                    __print(format_args!(
                        "[{:>7.3}] {}: {}:{} {} \n",
                        time_secs,
                        " warn".bright_yellow(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.yellow().bold()
                    ));
                },
                Level::Info => {
                    __print(format_args!(
                        "[{:>7.3}] {} \n",
                        time_secs,
                        content.white().bold()
                    ));
                },
                Level::Debug => {
                    __print(format_args!(
                        "[{:>7.3}] {}: {}:{} {} \n",
                        time_secs,
                        "debug".black(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content
                    ));
                },
                Level::Trace => {
                    __print(format_args!(
                        "[{:>7.3}] {}: {}:{} {} \n",
                        time_secs,
                        "trace".bright_purple(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.white().bold()
                    ));
                },
            }
        }
    }

    fn flush(&self) {}
}

pub fn install() {
    log::set_logger(&TTY).unwrap();
    log::set_max_level(TTY_LEVEL);
}

pub fn __print(args: fmt::Arguments<'_>) {
    if devices::tty::serial::ready() {
        devices::tty::serial::print(args);
    }

    if devices::tty::fb::ready() {
        devices::tty::fb::print(args);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::logging::__print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        use alloc::format;
        use owo_colors::OwoColorize;
        $crate::print!("{}\n", format!($($arg)*).red());
});
}
