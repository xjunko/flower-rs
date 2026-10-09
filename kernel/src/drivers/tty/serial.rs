use core::fmt::Write;

use spin::Mutex;
use x86_64::instructions::port::Port;

pub struct SerialPort {
    data: Port<u8>,
    interrupt: Port<u8>,
    fifo: Port<u8>,
    line: Port<u8>,
    modem: Port<u8>,
    status: Port<u8>,
}

impl SerialPort {
    // honestly this seems a little bit hacky
    // because if the status line is stuck it's most likely
    // that something terribly has gone wrong, but whatever
    // limit the wait anyway....
    const MAX_WAIT: usize = 100_000;

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

    fn wait_ready(&mut self) -> bool {
        for _ in 0..Self::MAX_WAIT {
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
pub static SERIAL: Mutex<SerialPort> =
    Mutex::new(SerialPort::new(_DEFAULT_COM_PORT));

pub fn install() { SERIAL.lock().init() }
