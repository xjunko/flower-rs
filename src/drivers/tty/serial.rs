use core::fmt::{self, Write};

use spin::Mutex;
use x86_64::instructions::{interrupts, port::Port};
pub struct SerialPort {
    data: Port<u8>,
    interrupt: Port<u8>,
    fifo: Port<u8>,
    line: Port<u8>,
    modem: Port<u8>,
    status: Port<u8>,
}

impl SerialPort {
    pub const fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            interrupt: Port::new(base + 1),
            fifo: Port::new(base + 2),
            line: Port::new(base + 3),
            modem: Port::new(base + 4),
            status: Port::new(base + 5),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.interrupt.write(0x00);
            self.line.write(0x80);
            self.data.write(0x03);
            self.interrupt.write(0x00);
            self.line.write(0x03);
            self.fifo.write(0xC7);
            self.modem.write(0x0B);
        }
    }

    // honestly this seems a little bit hacky
    // because if the status line is stuck it's most likely
    // that something terribly has gone wrong, but whatever
    // limit the wait anyway....
    fn wait_ready(&mut self) -> bool {
        for _ in 0..100_000 {
            unsafe {
                if (self.status.read() & 0x20) != 0 {
                    return true;
                }
            }
        }
        false
    }

    pub unsafe fn write(&mut self, byte: u8) {
        if self.wait_ready() {
            unsafe {
                self.data.write(byte);
            }
        }
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            unsafe { self.write(c as u8) };
        }
        Ok(())
    }
}

const _DEFAULT_COM_PORT: u16 = 0x3F8;
pub static SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort::new(_DEFAULT_COM_PORT));

// public APIs
pub fn install() {
    SERIAL.lock().init()
}

pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        let mut serial = SERIAL.lock();
        let _ = serial.write_fmt(args);

        if let Some(mut flanterm_context) = super::flanterm::get() {
            let _ = flanterm_context.write_fmt(args);
        }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::drivers::tty::serial::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::print!(
            "\x1b[32m[I]\x1b[0m [{}:{}] {}\n",
            file!(), line!(), format_args!($($arg)*)
        );
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::print!(
            "\x1b[32m[D]\x1b[0m [{}:{}] {}\n",
            file!(), line!(), format_args!($($arg)*)
        );
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::print!(
            "\x1b[33m[W]\x1b[0m [{}:{}] {}\n",
            file!(), line!(), format_args!($($arg)*)
        );
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::print!(
            "\x1b[31m[E]\x1b[0m [{}:{}] {}\n",
            file!(), line!(), format_args!($($arg)*)
        );
    };
}
