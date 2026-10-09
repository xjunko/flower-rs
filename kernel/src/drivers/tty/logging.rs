use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

use crate::println;

struct FlowerLogger;

impl log::Log for FlowerLogger {
    fn enabled(&self, _: &log::Metadata) -> bool { true }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let content = record.args();
            match record.level() {
                Level::Error => {
                    println!(
                        "{} {}:{} {}",
                        " Error ".white().bold().on_bright_red(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.white().bold().on_bright_red()
                    )
                },
                Level::Warn => {
                    println! {
                        "{} {}:{} {}",
                         " Warn  ".white().bold().on_bright_yellow(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.yellow().bold()
                    }
                },
                Level::Info => {
                    println!(
                        "{} {}",
                        " Info  ".white().bold().on_bright_cyan(),
                        content
                    )
                },
                Level::Debug => {
                    println!(
                        "{} {}:{} {}",
                        " Debug ".white().bold().on_bright_black(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content
                    );
                },
                Level::Trace => {
                    println!(
                        "{} {}:{} {}",
                        " Trace ".white().bold().on_bright_purple(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content
                    );
                },
            }
        }
    }

    fn flush(&self) {}
}

static LOG: FlowerLogger = FlowerLogger;
const LOG_LEVEL: LevelFilter = LevelFilter::Trace;

pub fn install() {
    log::set_logger(&LOG).unwrap();
    log::set_max_level(LOG_LEVEL);
}
