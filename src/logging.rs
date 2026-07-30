use alloc::fmt;

use log::{Level, LevelFilter};
use owo_colors::OwoColorize;

use crate::devices;

static TTY: SerialTty = SerialTty;
const TTY_LEVEL: LevelFilter = LevelFilter::Info;

struct SerialTty;

impl log::Log for SerialTty {
    fn enabled(&self, _metadata: &log::Metadata) -> bool { true }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let content = record.args();
            match record.level() {
                Level::Error => {
                    __print(format_args!(
                        "[{}] {}:{} {} \n",
                        "error".bright_red(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.red().bold()
                    ));
                },
                Level::Warn => {
                    __print(format_args!(
                        "[{}] {}:{} {} \n",
                        " warn".bright_yellow(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content.yellow().bold()
                    ));
                },
                Level::Info => {
                    __print(format_args!(
                        "[{}] {} \n",
                        " info".bright_green(),
                        content.white().bold()
                    ));
                },
                Level::Debug => {
                    __print(format_args!(
                        "[{}] {}:{} {} \n",
                        "debug".black(),
                        record.file().unwrap_or("unknown").bold(),
                        record.line().unwrap_or(0),
                        content
                    ));
                },
                Level::Trace => {
                    __print(format_args!(
                        "[{}] {}:{} {} \n",
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

    if devices::tty::terminal::ready() {
        devices::tty::terminal::print(args);
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
