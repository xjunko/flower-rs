use alloc::collections::vec_deque::VecDeque;
use alloc::string::String;
use core::arch::naked_asm;

pub use process::*;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::PageTableFlags;

use crate::arch::{self, gdt};
use crate::debug;
use crate::system::elf;
use crate::system::mem::vmm::AddressSpace;

pub mod process;
mod trampoline;

pub static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

pub struct Scheduler {
    processes: VecDeque<Process>,
    current: usize,
}

impl Scheduler {
    pub fn new() -> Self { Self { processes: VecDeque::new(), current: 0 } }

    /// adds a process to the scheduler.
    pub fn add(&mut self, process: Process) {
        if process.state != ProcessState::Ready
            && process.state != ProcessState::Running
        {
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
            if i != self.current
                && self.processes[i].state == ProcessState::Dead
            {
                debug!("reaping process {}", self.processes[i].name);
                self.processes.remove(i);
                if i < self.current {
                    self.current -= 1;
                }
            }
        }
    }

    /// wakes up any sleeping processes whose wake time has come.
    pub fn wake_sleeping(&mut self) {
        let ticks = arch::interrupts::get_ticks();
        for proc in self.processes.iter_mut() {
            if proc.state == ProcessState::Sleeping
                && let Some(wake_at) = proc.wake_at
                && ticks >= wake_at
            {
                proc.state = ProcessState::Ready;
                proc.wake_at = None;
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

        (old_sp, new_sp, cr3_to_load, kernel_stack)
    }
}

/// schedules the process
pub fn schedule() {
    interrupts::without_interrupts(|| {
        let ctx_change = {
            let mut guard = SCHEDULER.lock();
            if let Some(sched) = guard.as_mut() {
                sched.reap();
                sched.wake_sleeping();
                sched.next().map(|next| sched.switch_to(next))
            } else {
                panic!("trying to schedule while not initialized!");
            }
        };

        if let Some((old_sp, new_sp, new_cr3, kernel_stack)) = ctx_change {
            if kernel_stack != 0 {
                gdt::set_kernel_stack(VirtAddr::new(kernel_stack));
            }
            unsafe {
                Scheduler::switch_context(old_sp, new_sp, new_cr3, kernel_stack)
            }
        }
    });
}

/// spawns a new process with the given entry point and name.
pub fn spawn(name: &str, entry: fn()) {
    let new_process = Process::new(name, entry);
    debug!("created process {}", new_process.name);
    interrupts::without_interrupts(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            debug!("adding process {} to scheduler", new_process.name);
            sched.add(new_process);
        }
    });
}

/// spawns an elf process with the given name and elf bytes.
pub fn spawn_elf(name: &str, elf_data: &[u8]) -> Result<u64, &'static str> {
    let address_space = AddressSpace::new()?;
    let loaded = elf::load_into(elf_data, &address_space)?;

    if !address_space.is_mapped(VirtAddr::new(loaded.entry & !0xFFF)) {
        return Err("entry point is not mapped");
    }

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE
        | PageTableFlags::NO_EXECUTE;

    let user_stack_top_page: u64 = 0x0007_FFFF_F000;
    let user_stack_pages = 4;

    for i in 0..user_stack_pages {
        let page_addr = user_stack_top_page - (i * 0x1000);
        address_space.map_page_alloc(VirtAddr::new(page_addr), flags)?;
    }

    let user_stack_top = user_stack_top_page + 4096 - 8;
    let proc =
        Process::new_user(name, address_space, loaded.entry, user_stack_top);
    let proc_id = proc.id;
    debug!(
        "created process {} with entry point {:#x}",
        proc.name, loaded.entry
    );

    if let Some(sched) = SCHEDULER.lock().as_mut() {
        sched.add(proc);
    }

    Ok(proc_id)
}

/// sleeps the current process for the given number of ticks.
pub fn sleep(ticks: u64) {
    let wake_at = arch::ticks() + ticks;
    interrupts::without_interrupts(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                proc.state = ProcessState::Sleeping;
                proc.wake_at = Some(wake_at);
            } else {
                panic!("no current process to sleep!");
            }
        } else {
            panic!("trying to sleep while not initialized!");
        }
    });
    schedule();
}

/// exits the current process.
pub fn exit() {
    arch::syscalls::restore_kernel_gs_base();

    interrupts::without_interrupts(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            sched.processes[sched.current].state = ProcessState::Dead;
        }
    });
    schedule();
    unreachable!();
}

/// returns the current pid
pub fn current() -> Option<usize> {
    interrupts::without_interrupts(|| {
        SCHEDULER.lock().as_ref().map(|sched| sched.current)
    })
}

/// gets the current process name
pub fn name() -> String {
    interrupts::without_interrupts(|| {
        SCHEDULER
            .lock()
            .as_ref()
            .map(|sched| sched.processes[sched.current].name.clone())
            .unwrap_or(String::from("undefined"))
    })
}

/// installs the scheduler, initializing the null process and adding it to the scheduler.
pub fn install() {
    let mut scheduler = Scheduler::new();
    scheduler.add(null_process());
    interrupts::without_interrupts(|| {
        *SCHEDULER.lock() = Some(scheduler);
    });
}
