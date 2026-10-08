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

use spin::Mutex;

use crate::proc::process::{Process, ProcessState};

pub struct Scheduler {
    pub processes: VecDeque<Arc<Mutex<Process>>>,
    pub current: Mutex<usize>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { processes: VecDeque::new(), current: 0.into() }
    }

    pub fn add(&mut self, process: Process) {
        let process_arc = Arc::new(Mutex::new(process));
        self.processes.push_back(process_arc);
    }

    pub fn current(&mut self) -> Option<Arc<Mutex<Process>>> {
        let current_index = *self.current.lock();
        self.processes.get(current_index).cloned()
    }

    pub fn next_idx(&self) -> Option<usize> {
        let length = self.processes.len();

        for i in 1..length {
            let idx = (*self.current.lock() + i) % length;
            let cur_process = self.processes[idx].lock();
            let cur_state = cur_process.state.lock();

            if *cur_state == ProcessState::Ready {
                return Some(idx);
            }
        }

        return None;
    }

    pub fn switch_to(&mut self, next_idx: usize) -> (*mut u64, u64, u64) {
        let current = *self.current.lock();
        *self.current.lock() = next_idx;

        let mut cur_proc = self.processes[current].lock();
        let next_proc = self.processes[next_idx].lock();

        if *cur_proc.state.lock() == ProcessState::Running {
            *cur_proc.state.lock() = ProcessState::Ready;
        }

        *next_proc.state.lock() = ProcessState::Running;

        let old_sp = &mut cur_proc._stack_ptr as *mut u64;
        let new_sp = next_proc._stack_ptr;
        let new_stack_top = next_proc._stack_top;

        drop(next_proc);
        drop(cur_proc);

        (old_sp, new_sp, new_stack_top)
    }
}
