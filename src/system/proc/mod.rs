use alloc::collections::vec_deque::VecDeque;
use core::arch::naked_asm;

pub use process::*;
use spin::Mutex;
use x86_64::{VirtAddr, instructions::interrupts};

use crate::{arch::gdt, debug};
pub mod process;
mod trampoline;

pub static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

pub struct Scheduler {
    processes: VecDeque<Process>,
    current: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            processes: VecDeque::new(),
            current: 0,
        }
    }

    /// adds a process to the scheduler.
    pub fn add(&mut self, process: Process) {
        if process.state != ProcessState::Ready {
            panic!(
                "cannot add process {} to scheduler because it is not ready",
                process.name
            );
        }

        self.processes.push_back(process);
    }

    /// finds the next ready process to run, returning its index if found.
    fn next(&self) -> Option<usize> {
        let length = self.processes.len();
        for i in 1..length {
            let idx = (self.current + i) % length;
            if self.processes[idx].state == ProcessState::Ready {
                return Some(idx);
            }
        }
        None
    }

    /// reaps any dead processes, removing them from the scheduler.
    fn reap(&mut self) {
        let mut i = self.processes.len();
        while i > 0 {
            i -= 1;
            if i != self.current && self.processes[i].state == ProcessState::Dead {
                debug!("reaping process {}", self.processes[i].name);
                self.processes.remove(i);
                if i < self.current {
                    self.current -= 1;
                }
            }
        }
    }

    /// returns a mutable reference to the current process, if any.
    fn current(&mut self) -> Option<&mut Process> {
        self.processes.get_mut(self.current)
    }

    #[unsafe(naked)]
    /// performs a context switch to the process with the given pid, returning the old stack pointer and the new stack pointer.
    unsafe extern "C" fn switch_context(
        old_sp: *mut u64,
        new_sp: u64,
        new_cr3: u64,
        new_stack_top: u64,
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
    fn switch_to(&mut self, next: usize) -> (*mut u64, u64, u64, u64) {
        let current = self.current;

        self.current = next;
        if self.processes[current].state == ProcessState::Running {
            self.processes[current].state = ProcessState::Ready;
        }
        self.processes[next].state = ProcessState::Running;

        let old_sp = &mut self.processes[current].stack_ptr as *mut u64;
        let new_sp = self.processes[next].stack_ptr;

        let old_cr3 = self.processes[current].cr3;
        let new_cr3 = self.processes[next].cr3;
        let cr3_to_load = if old_cr3 != new_cr3 { new_cr3 } else { 0 };

        let kernel_stack = self.processes[next].kernel_stack_top;

        if kernel_stack != 0 {
            gdt::set_kernel_stack(VirtAddr::new(kernel_stack));
        }

        (old_sp, new_sp, cr3_to_load, kernel_stack)
    }
}

/// spawns a new process with the given entry point and name.
pub fn spawn(name: &str, entry: fn()) {
    let new_process = Process::new(name, entry);
    debug!("created process {}", new_process.name);
    if let Some(sched) = SCHEDULER.lock().as_mut() {
        debug!("adding process {} to scheduler", new_process.name);
        sched.add(new_process);
    }
}

/// exits the current process.
pub fn exit() {
    interrupts::without_interrupts(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            debug!("exiting process {}", sched.processes[sched.current].name);
            sched.processes[sched.current].state = ProcessState::Dead;
        }
    });
    schedule();
    crate::arch::halt();
}

/// schedules the process
pub fn schedule() {
    let ctx_change = interrupts::without_interrupts(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            sched.reap();
            sched.next().map(|next| sched.switch_to(next))
        } else {
            None
        }
    });

    if let Some((old_sp, new_sp, new_cr3, kernel_stack)) = ctx_change {
        unsafe { Scheduler::switch_context(old_sp, new_sp, new_cr3, kernel_stack) }
    }
}

/// returns the current pid
pub fn current() -> Option<usize> {
    SCHEDULER.lock().as_ref().map(|sched| sched.current)
}

pub fn install() {
    let mut scheduler = Scheduler::new();
    scheduler.add(null_process());
    *SCHEDULER.lock() = Some(scheduler);
}
