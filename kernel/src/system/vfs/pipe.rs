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

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

use spin::Mutex;

use crate::system::posix::errno::{KError, KResult};

const CAPACITY: usize = 8192;

struct KPipeState {
    buffer: VecDeque<u8>,
    read_closed: bool,
    write_closed: bool,
}

pub struct KPipe {
    state: Mutex<KPipeState>,
}

pub struct KPipeEnd {
    pipe: Arc<KPipe>,
    write: bool,
    nonblock: AtomicBool,
}

impl KPipeEnd {
    pub fn read(&self, buf: &mut [u8]) -> KResult<usize> {
        if self.write {
            return Err(KError::EBADF);
        }

        if buf.is_empty() {
            return Ok(0);
        }

        let mut state = self.pipe.state.lock();

        if state.buffer.is_empty() {
            return if state.write_closed {
                Ok(0)
            } else {
                Err(KError::EAGAIN)
            };
        }

        let count = buf.len().min(state.buffer.len());
        for slot in buf.iter_mut().take(count) {
            *slot = state.buffer.pop_front().unwrap_or(0);
        }

        Ok(count)
    }

    pub fn write(&self, buf: &[u8]) -> KResult<usize> {
        if !self.write {
            return Err(KError::EBADF);
        }

        if buf.is_empty() {
            return Ok(0);
        }

        let mut state = self.pipe.state.lock();

        if state.read_closed {
            return Err(KError::EPIPE);
        }

        if state.buffer.len() >= CAPACITY {
            return Err(KError::EAGAIN);
        }

        let count = buf.len().min(CAPACITY - state.buffer.len());
        state.buffer.extend(buf.iter().copied().take(count));

        Ok(count)
    }

    pub fn is_nonblock(&self) -> bool { self.nonblock.load(Ordering::Relaxed) }

    pub fn set_nonblock(&self, nonblock: bool) {
        self.nonblock.store(nonblock, Ordering::Relaxed);
    }
}

impl Drop for KPipeEnd {
    fn drop(&mut self) {
        let mut state = self.pipe.state.lock();

        if self.write {
            state.write_closed = true;
        } else {
            state.read_closed = true;
        }
    }
}

pub fn create(nonblock: bool) -> (Arc<KPipeEnd>, Arc<KPipeEnd>) {
    let pipe = Arc::new(KPipe {
        state: Mutex::new(KPipeState {
            buffer: VecDeque::with_capacity(CAPACITY),
            read_closed: false,
            write_closed: false,
        }),
    });

    let read = Arc::new(KPipeEnd {
        pipe: Arc::clone(&pipe),
        write: false,
        nonblock: AtomicBool::new(nonblock),
    });

    let write = Arc::new(KPipeEnd {
        pipe,
        write: true,
        nonblock: AtomicBool::new(nonblock),
    });

    (read, write)
}
