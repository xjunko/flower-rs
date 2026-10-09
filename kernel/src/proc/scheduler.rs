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

use crate::arch::{Paging, Processor};
use crate::memory::vmm::AddressSpace;
use crate::proc::process::{Process, ProcessState};

type Root = <Processor as Paging>::Root;

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

    pub fn first_idx(&self) -> Option<usize> {
        self.processes.iter().position(|process| {
            *process.lock().state.lock() == ProcessState::Ready
        })
    }

    pub fn activate(&mut self, next_idx: usize) -> (u64, u64, Option<Root>) {
        *self.current.lock() = next_idx;

        let next_proc = self.processes[next_idx].lock();
        *next_proc.state.lock() = ProcessState::Running;

        let kernel_root = AddressSpace::kernel().root();
        let new_root = next_proc
            .address_space
            .lock()
            .as_ref()
            .map(AddressSpace::root)
            .unwrap_or(kernel_root);

        (
            next_proc._stack_ptr,
            next_proc._stack_top,
            (new_root != kernel_root).then_some(new_root),
        )
    }

    pub fn reap(&mut self) {
        let mut i = self.processes.len();
        let mut current_idx = *self.current.lock();
        while i > 0 {
            i -= 1;

            let reapable = {
                let proc = self.processes[i].lock();
                let state = proc.state.lock();
                *state == ProcessState::Dead
                    || (*state == ProcessState::Zombie
                        && proc.parent_id.lock().is_none())
            };

            if i != current_idx && reapable {
                log::trace!(
                    "reaping process: {:?}",
                    self.processes[i].lock().name.lock()
                );
                self.processes.remove(i);

                if i < current_idx {
                    current_idx -= 1;
                }
            }
        }
        *self.current.lock() = current_idx;
    }

    pub fn switch_to(
        &mut self,
        next_idx: usize,
    ) -> (*mut u64, u64, u64, Option<Root>) {
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

        let root_to_load = {
            let kernel_root = AddressSpace::kernel().root();
            let old_root = cur_proc
                .address_space
                .lock()
                .as_ref()
                .map(AddressSpace::root)
                .unwrap_or(kernel_root);

            let new_root = next_proc
                .address_space
                .lock()
                .as_ref()
                .map(AddressSpace::root)
                .unwrap_or(kernel_root);

            (old_root != new_root).then_some(new_root)
        };

        drop(next_proc);
        drop(cur_proc);

        (old_sp, new_sp, new_stack_top, root_to_load)
    }
}
