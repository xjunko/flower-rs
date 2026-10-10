use core::fmt::Write;

use spin::Mutex;

pub struct SerialPort {
    data: u16,
    interrupt: u16,
    fifo: u16,
    line: u16,
    modem: u16,
    status: u16,
}

impl SerialPort {
    // honestly this seems a little bit hacky
    // because if the status line is stuck it's most likely
    // that something terribly has gone wrong, but whatever
    // limit the wait anyway....
    const MAX_WAIT: usize = 100_000;

    pub const fn new(base: u16) -> Self {
        Self {
            data: base,
            interrupt: base + 1,
            fifo: base + 2,
            line: base + 3,
            modem: base + 4,
            status: base + 5,
        }
    }

    pub fn init(&mut self) {
        crate::arch::port::write::<u8>(self.interrupt, 0x00);
        crate::arch::port::write::<u8>(self.line, 0x80);
        crate::arch::port::write::<u8>(self.data, 0x03);
        crate::arch::port::write::<u8>(self.interrupt, 0x00);
        crate::arch::port::write::<u8>(self.line, 0x03);
        crate::arch::port::write::<u8>(self.fifo, 0xC7);
        crate::arch::port::write::<u8>(self.modem, 0x0B);
    }

    fn wait_ready(&mut self) -> bool {
        for _ in 0..Self::MAX_WAIT {
            if (crate::arch::port::read::<u8>(self.status) & 0x20) != 0 {
                return true;
            }
        }
        false
    }

    pub unsafe fn write(&mut self, byte: u8) {
        if self.wait_ready() {
            crate::arch::port::write::<u8>(self.data, byte);
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
