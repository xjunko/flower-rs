use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

use crate::arch::{Processor, Timers};
use crate::println;

struct FlowerLogger;

impl log::Log for FlowerLogger {
    fn enabled(&self, _: &log::Metadata) -> bool { true }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let content = record.args();
            let time_secs = Processor::timer_get_ticks() as f64 / 1000.0;

            match record.level() {
                Level::Error => {
                    println!(
                        "[{:>7.3}] {}: {}:{} {}",
                        time_secs,
                        "error".bright_red(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.red().bold()
                    );
                },
                Level::Warn => {
                    println!(
                        "[{:>7.3}] {}: {}:{} {}",
                        time_secs,
                        " warn".bright_yellow(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.yellow().bold()
                    );
                },
                Level::Info => {
                    println!("[{:>7.3}] {}", time_secs, content.white().bold());
                },
                Level::Debug => {
                    println!(
                        "[{:>7.3}] {}: {}:{} {}",
                        time_secs,
                        "debug".black(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content
                    );
                },
                Level::Trace => {
                    println!(
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

static LOG: FlowerLogger = FlowerLogger;
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

pub fn install() {
    log::set_logger(&LOG).unwrap();
    log::set_max_level(LOG_LEVEL);
}
