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

use spin::Mutex;

const MAX_SUBSCRIBERS: usize = 16;

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
