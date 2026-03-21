pub mod process;
mod trampoline;

use alloc::collections::vec_deque::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use core::arch::naked_asm;

pub use process::*;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::registers::model_specific::FsBase;
use x86_64::structures::paging::PageTableFlags;

use crate::arch::{self};
use crate::system::elf;
use crate::system::mem::vmm::AddressSpace;
use crate::system::vfs::{FdTable, VFSError, VFSResult};
use crate::{debug, system};

pub static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

const USER_STACK_TOP_PAGE: u64 = 0x7_FFFF_F000;
const USER_STACK_PAGES: u64 = 4;
const USER_STACK_INITIAL_SLACK: u64 = 0x100;
const PAGE_SIZE: u64 = system::mem::PAGE_SIZE as u64;

pub struct Scheduler {
    processes: VecDeque<Arc<Mutex<Process>>>,
    current: usize,
}

impl Scheduler {
    pub fn new() -> Self { Self { processes: VecDeque::new(), current: 0 } }

    /// returns a mutable reference to the current process, if any.
    fn current(&mut self) -> Option<Arc<Mutex<Process>>> {
        self.processes.get(self.current).cloned()
    }

    /// finds the next ready process to run, returning its index if found.
    fn next(&self) -> Option<usize> {
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
    fn reap(&mut self) {
        let mut i = self.processes.len();
        while i > 0 {
            i -= 1;
            if i != self.current
                && self.processes[i].lock().state == ProcessState::Dead
            {
                debug!("reaping process {}", self.processes[i].lock().name);
                self.processes.remove(i);
                if i < self.current {
                    self.current -= 1;
                }
            }
        }
    }

    /// awakens any sleeping processes whose wake time has passed, setting them to ready.
    fn awaken(&mut self) {
        let ticks = arch::ticks();
        for proc in self.processes.iter_mut() {
            let mut proc = proc.lock();
            if proc.state == ProcessState::Sleeping
                && let Some(wake_at) = proc.wake_at
                && ticks >= wake_at
            {
                proc.state = ProcessState::Ready;
                proc.wake_at = None;
            }
        }
    }

    /// performs a context switch to the process with the given pid, returning the old stack pointer and the new stack pointer.
    #[unsafe(naked)]
    unsafe extern "C" fn switch_context(
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
    fn switch_to(&mut self, next: usize) -> (*mut u64, u64, u64) {
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
            next_proc.switch_stack();
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

/// schedules the process
pub fn schedule() {
    interrupts::without_interrupts(|| {
        let ctx_change = {
            let mut guard = SCHEDULER.lock();
            if let Some(sched) = guard.as_mut() {
                sched.reap();
                sched.awaken();
                sched.next().map(|next| sched.switch_to(next))
            } else {
                panic!("trying to schedule while not initialized!");
            }
        };

        if let Some((old_sp, new_sp, new_cr3)) = ctx_change {
            unsafe { Scheduler::switch_context(old_sp, new_sp, new_cr3) }
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
        | PageTableFlags::USER_ACCESSIBLE;

    let mut user_heap = loaded.entry + loaded.size as u64;
    {
        user_heap = (user_heap + PAGE_SIZE - 1) & !0xFFF; // align to page
        address_space.map_page_alloc(VirtAddr::new(user_heap), flags)?;
        user_heap += PAGE_SIZE; // bump past the pre-mapped page
    }

    for i in 0..USER_STACK_PAGES {
        let page_addr = USER_STACK_TOP_PAGE - (i * PAGE_SIZE);
        address_space.map_page_alloc(VirtAddr::new(page_addr), flags)?;
    }

    let stack_low =
        USER_STACK_TOP_PAGE + PAGE_SIZE - (USER_STACK_PAGES * PAGE_SIZE);
    let user_stack_top =
        (USER_STACK_TOP_PAGE + PAGE_SIZE - USER_STACK_INITIAL_SLACK) & !0xF;
    debug_assert!(
        user_stack_top >= stack_low
            && user_stack_top < USER_STACK_TOP_PAGE + PAGE_SIZE
    );
    let proc = Process::new_user(
        name,
        address_space,
        loaded.entry,
        user_stack_top,
        user_heap,
    );
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

/// loops over the file descriptors of the current process
pub fn with_fd_table<F, R>(f: F) -> VFSResult<R>
where F: FnOnce(&mut FdTable) -> VFSResult<R> {
    let mut guard = SCHEDULER.lock();
    let sched = guard.as_mut().ok_or(VFSError::IOError)?;
    let task = sched.current().ok_or(VFSError::IOError)?;
    f(&mut task.lock().fds)
}

/// sleeps the current process for the given number of milliseconds.
pub fn sleep(millis: u64) {
    let wake_at = arch::ticks() + millis;

    interrupts::without_interrupts(|| {
        system::syscalls::write_cpu_context();
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                let mut proc = proc.lock();
                proc.wake_at = Some(wake_at);
                proc.state = ProcessState::Sleeping;
            } else {
                panic!("trying to sleep while no process is running!");
            }
        } else {
            panic!("trying to sleep while not initialized!");
        }
    });
    schedule();
}

/// exits the current process.
pub fn exit() {
    interrupts::without_interrupts(|| {
        system::syscalls::write_cpu_context();
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                let mut proc = proc.lock();
                proc.state = ProcessState::Dead;
            } else {
                panic!("trying to exit while no process is running!");
            }
        } else {
            panic!("trying to exit while not initialized!");
        }
    });
    schedule();
    unreachable!();
}

/// returns the current process
pub fn current() -> Option<Arc<Mutex<Process>>> {
    interrupts::without_interrupts(|| {
        SCHEDULER.lock().as_mut().and_then(|sched| sched.current())
    })
}

/// returns the current pid
pub fn current_pid() -> Option<usize> {
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
            .map(|sched| sched.processes[sched.current].lock().name.clone())
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
