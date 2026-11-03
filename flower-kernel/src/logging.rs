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

use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

use crate::arch::{Processor, Timers};

macro_rules! output {
    ($($arg:tt)*) => {
        crate::drivers::tty::_log(format_args!(
            "{}\n",
            format_args!($($arg)*)
        ))
    };
}

struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, _: &log::Metadata) -> bool { true }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let content = record.args();
            let time_secs = Processor::timer_get_ns() as f64 / 1_000_000_000.0;

            match record.level() {
                Level::Error => {
                    output!(
                        "[{:>7.3}] {}: {}:{} {}",
                        time_secs,
                        "error".bright_red(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.red().bold()
                    );
                },
                Level::Warn => {
                    output!(
                        "[{:>7.3}] {}: {}:{} {}",
                        time_secs,
                        " warn".bright_yellow(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.yellow().bold()
                    );
                },
                Level::Info => {
                    output!("[{:>7.3}] {}", time_secs, content.white().bold());
                },
                Level::Debug => {
                    output!(
                        "[{:>7.3}] {}: {}:{} {}",
                        time_secs,
                        "debug".black(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content
                    );
                },
                Level::Trace => {
                    output!(
                        "[{:>7.3}] {}: {}:{} {}",
                        time_secs,
                        "trace".bright_purple(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.white().bold()
                    );
                },
            }
        }
    }

    fn flush(&self) {}
}

static LOG: KernelLogger = KernelLogger;
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

pub fn install() {
    log::set_logger(&LOG).unwrap();
    log::set_max_level(LOG_LEVEL);
}
