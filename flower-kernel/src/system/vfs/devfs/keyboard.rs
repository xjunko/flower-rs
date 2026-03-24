use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::ToString;

use spin::Mutex;

use crate::drivers::ps2::keyboard::{KEYBOARD, KeyEvent, KeyboardSubscriber};
use crate::system::vfs::devfs::{DevFS, DevFile};

static KB_BUFFER: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());

struct DevFSKeyboard;

impl KeyboardSubscriber for DevFSKeyboard {
    fn on_key_event(&mut self, event: KeyEvent) {
        if let KeyEvent::Ascii(byte) = event {
            KB_BUFFER.lock().push_back(byte);
        }
    }
}

fn kb_read(_offset: usize, buf: &mut [u8]) -> usize {
    let mut queue = KB_BUFFER.lock();
    let mut read = 0;

    for out in buf.iter_mut() {
        let Some(byte) = queue.pop_front() else {
            break;
        };

        *out = byte;
        read += 1;
    }

    read
}

fn kb_write(_buf: &[u8]) -> usize { 0 }

pub fn install(dev: &mut DevFS) {
    let subscriber = Box::leak(Box::new(DevFSKeyboard));
    KEYBOARD.lock().subscribe(subscriber);

    dev.bind(DevFile::new(
        "/keyboard".to_string(),
        Some(kb_read),
        Some(kb_write),
    ));
}
