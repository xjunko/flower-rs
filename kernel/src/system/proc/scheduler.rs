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

use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;

use spin::Mutex;

use crate::arch::{self, Multitasking, Processor};
use crate::system::proc::{Process, ProcessState};

pub struct Scheduler {
    pub processes: VecDeque<Arc<Mutex<Process>>>,
    pub current: usize,
}

impl Scheduler {
    pub fn new() -> Self { Self { processes: VecDeque::new(), current: 0 } }

    /// returns the current idx of the running process
    pub fn current_idx(&self) -> usize { self.current }

    /// returns a mutable reference to the current process, if any.
    pub fn current(&mut self) -> Option<Arc<Mutex<Process>>> {
        self.processes.get(self.current).cloned()
    }

    /// finds the next ready process to run, returning its index if found.
    pub fn next(&self) -> Option<usize> {
        let length = self.processes.len();
        for i in 1..length {
            let idx = (self.current + i) % length;
            if self.processes[idx].lock().state() == ProcessState::Ready {
                return Some(idx);
            }
        }
        None
    }

    /// reaps any dead processes, removing them from the scheduler.
    pub fn reap(&mut self) {
        let mut i = self.processes.len();
        while i > 0 {
            i -= 1;
            let reapable = {
                let proc = self.processes[i].lock();
                proc.state() == ProcessState::Dead
                    || (proc.state() == ProcessState::Zombie
                        && proc.parent_id().is_none())
            };

            if i != self.current && reapable {
                log::trace!(
                    "reaping process {}",
                    self.processes[i].lock().name()
                );
                self.processes.remove(i);
                if i < self.current {
                    self.current -= 1;
                }
            }
        }
    }

    /// awakens any sleeping processes whose wake time has passed, setting them to ready.
    pub fn awaken(&mut self) {
        let nanos = arch::timers::ns();
        for proc in self.processes.iter_mut() {
            let mut proc = proc.lock();

            match proc.state() {
                ProcessState::Sleeping(wake_at_ns) => {
                    if nanos > wake_at_ns {
                        log::trace!(
                            "awakening process {} (woke at {}, current ticks {})",
                            proc.name(),
                            wake_at_ns,
                            nanos
                        );
                        proc.set_state(ProcessState::Ready);
                    }
                },
                _ => continue,
            }
        }
    }

    /// switches the context
    pub fn switch_context(old_sp: *mut u64, new_sp: u64, new_root: u64) {
        Processor::switch_context(
            old_sp,
            new_sp,
            if new_root == 0 { None } else { Some(new_root) },
        );
    }

    /// switches to the process with the given pid, returning the old stack pointer and the new stack pointer.
    pub fn switch_to(&mut self, next: usize) -> (*mut u64, u64, u64) {
        let current = self.current;

        self.current = next;

        let mut current_proc = self.processes[current].lock();
        let mut next_proc = self.processes[next].lock();

        if current_proc.state() == ProcessState::Running {
            current_proc.set_state(ProcessState::Ready);
        }
        next_proc.set_state(ProcessState::Running);

        let old_sp = current_proc.kernel_stack_mut().pointer_mut() as *mut u64;
        let new_sp = next_proc.kernel_stack().pointer();

        let old_cr3 = current_proc.address_space_root();
        let new_cr3 = next_proc.address_space_root();
        let cr3_to_load = if old_cr3 != new_cr3 { new_cr3 } else { 0 };

        #[cfg(target_arch = "x86_64")]
        // TODO: figure out a way to abstract this away
        {
            use x86_64::VirtAddr;
            use x86_64::registers::model_specific::FsBase;

            if current_proc.is_user() {
                current_proc.set_fsbase(FsBase::read().as_u64());
            }

            if next_proc.valid_stack() {
                unsafe {
                    next_proc.switch_stack();
                }
            }

            if next_proc.is_user() {
                FsBase::write(VirtAddr::new(next_proc.fsbase()));
            } else {
                FsBase::write(VirtAddr::new(0));
            }
        }

        (old_sp, new_sp, cr3_to_load)
    }

    /// adds a process to the scheduler.
    pub fn add(&mut self, process: Process) {
        let process = Arc::new(Mutex::new(process));
        if process.lock().state() != ProcessState::Ready
            && process.lock().state() != ProcessState::Running
        {
            panic!(
                "cannot add process {} to scheduler because it is not ready",
                process.lock().name()
            );
        }

        self.processes.push_back(process);
    }
}
