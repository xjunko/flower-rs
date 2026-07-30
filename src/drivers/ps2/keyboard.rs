use spin::{Lazy, Mutex};
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

use crate::arch::apic;
use crate::drivers::ps2::keyboard_defs::{
    scancode_to_ascii, scancode_to_keycode,
};

const MAX_SUBSCRIBERS: usize = 16;
const KB_DEVICE: u16 = 0x60;
const KB_PENDING: u16 = 0x64;

#[derive(Copy, Clone, Debug)]
pub enum KeyEvent {
    Press(u8),
    Release(u8),
    Ascii(u8),
}

pub trait KeyboardSubscriber: Send {
    fn on_key_event(&mut self, event: KeyEvent);
}

pub struct KeyboardPublisher {
    subscribers:
        Mutex<[Option<&'static mut dyn KeyboardSubscriber>; MAX_SUBSCRIBERS]>,
}

impl KeyboardPublisher {
    pub const fn new() -> Self {
        Self { subscribers: Mutex::new([const { None }; MAX_SUBSCRIBERS]) }
    }

    pub fn subscribe(&self, subscriber: &'static mut dyn KeyboardSubscriber) {
        let mut subscribers = self.subscribers.lock();
        for slot in subscribers.iter_mut() {
            if slot.is_none() {
                *slot = Some(subscriber);
                return;
            }
        }
        panic!("too many keyboard subscribers");
    }

    pub fn publish(&self, event: KeyEvent) {
        let mut subscribers = self.subscribers.lock();
        for subscriber in subscribers.iter_mut().flatten() {
            subscriber.on_key_event(event);
        }
    }
}
pub static KEYBOARD: Lazy<Mutex<KeyboardPublisher>> =
    Lazy::new(|| Mutex::new(KeyboardPublisher::new()));

static SHIFT_PRESSED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub extern "x86-interrupt" fn keyboard_interrupt_handler(
    _frame: InterruptStackFrame,
) {
    let mut pending_port: Port<u8> = Port::new(KB_PENDING);
    let mut data_port: Port<u8> = Port::new(KB_DEVICE);

    let pending = unsafe { pending_port.read() };
    if pending & 0x1 == 0 {
        apic::eoi();
        return;
    }

    let scancode = unsafe { data_port.read() };

    let mut shift_pressed = SHIFT_PRESSED.lock();
    if scancode == 0x2A || scancode == 0x36 {
        *shift_pressed = true;
    } else if scancode == 0xAA || scancode == 0xB6 {
        *shift_pressed = false;
    }

    let keycode = scancode_to_keycode(scancode & 0x7F);
    let event = if scancode & 0x80 == 0 {
        KeyEvent::Press(keycode as u8)
    } else {
        KeyEvent::Release(keycode as u8)
    };
    KEYBOARD.lock().publish(event);

    if scancode & 0x80 == 0
        && let Some(ascii) = scancode_to_ascii(scancode, *shift_pressed)
    {
        KEYBOARD.lock().publish(KeyEvent::Ascii(ascii));
    }

    apic::eoi();
}

const MAX_DRAIN: usize = 32;

pub fn install() {
    let mut pending_port: Port<u8> = Port::new(KB_PENDING);
    let mut data_port: Port<u8> = Port::new(KB_DEVICE);

    // optimally this should get all the
    // pending scancodes cleared out.
    for _ in 0..MAX_DRAIN {
        if unsafe { pending_port.read() } & 0x1 == 0 {
            break;
        }
        let _ = unsafe { data_port.read() };
    }
    log::debug!("ps2::keyboard installed!");
}
