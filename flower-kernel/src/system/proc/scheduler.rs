use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use core::arch::naked_asm;

use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::registers::model_specific::FsBase;

use crate::arch;
use crate::system::proc::{Process, ProcessLevel, ProcessState};

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
            if self.processes[idx].lock().state == ProcessState::Ready {
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
            if i != self.current
                && self.processes[i].lock().state == ProcessState::Dead
            {
                log::trace!(
                    "reaping process {}",
                    self.processes[i].lock().name
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
        let ticks = arch::ticks();
        for proc in self.processes.iter_mut() {
            let mut proc = proc.lock();
            if proc.state == ProcessState::Sleeping
                && let Some(wake_at) = proc.wake_at
                && ticks >= wake_at
            {
                log::trace!(
                    "awakening process {} (woke at {}, current ticks {})",
                    proc.name,
                    wake_at,
                    ticks
                );
                proc.state = ProcessState::Ready;
                proc.wake_at = None;
            }
        }
    }

    /// performs a context switch to the process with the given pid, returning the old stack pointer and the new stack pointer.
    #[unsafe(naked)]
    pub unsafe extern "C" fn switch_context(
        old_sp: *mut u64,
        new_sp: u64,
        new_cr3: u64,
    ) {
        naked_asm!(
            "push rbp",
            "push rbx",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov [rdi], rsp",
            "test rdx, rdx",
            "jz 2f",
            "mov cr3, rdx",
            "2:",
            "mov rsp, rsi",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp",
            "ret",
        );
    }

    /// switches to the process with the given pid, returning the old stack pointer and the new stack pointer.
    pub fn switch_to(&mut self, next: usize) -> (*mut u64, u64, u64) {
        let current = self.current;

        self.current = next;

        let mut current_proc = self.processes[current].lock();
        let mut next_proc = self.processes[next].lock();

        if current_proc.state == ProcessState::Running {
            current_proc.state = ProcessState::Ready;
        }
        next_proc.state = ProcessState::Running;

        let old_sp = &mut current_proc.stack_ptr as *mut u64;
        let new_sp = next_proc.stack_ptr;

        let old_cr3 = current_proc.cr3;
        let new_cr3 = next_proc.cr3;
        let cr3_to_load = if old_cr3 != new_cr3 { new_cr3 } else { 0 };

        if current_proc.level == ProcessLevel::RING3 {
            current_proc._fsbase = FsBase::read().as_u64();
        }

        if next_proc.valid_stack() {
            unsafe {
                next_proc.switch_stack();
            }
        }

        if next_proc.level == ProcessLevel::RING3 {
            FsBase::write(VirtAddr::new(next_proc._fsbase));
        } else {
            FsBase::write(VirtAddr::new(0));
        }

        (old_sp, new_sp, cr3_to_load)
    }

    /// adds a process to the scheduler.
    pub fn add(&mut self, process: Process) {
        let process = Arc::new(Mutex::new(process));
        if process.lock().state != ProcessState::Ready
            && process.lock().state != ProcessState::Running
        {
            panic!(
                "cannot add process {} to scheduler because it is not ready",
                process.lock().name
            );
        }

        self.processes.push_back(process);
    }
}
