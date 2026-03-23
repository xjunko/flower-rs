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
