mod exit;
mod process;
mod scheduler;
mod sleep;
mod trampoline;

pub mod user;

use alloc::string::String;
use alloc::sync::Arc;

pub use process::*;
use spin::Mutex;
use x86_64::instructions::interrupts;

pub use self::exit::exit;
pub use self::sleep::sleep;
use crate::system::proc::scheduler::Scheduler;
use crate::system::vfs::{FdTable, VFSError, VFSResult};

pub static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

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
    log::debug!("created process {}", new_process.name);
    interrupts::without_interrupts(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            log::debug!("adding process {} to scheduler", new_process.name);
            sched.add(new_process);
        }
    });
}

/// loops over the file descriptors of the current process
pub fn with_fd_table<F, R>(f: F) -> VFSResult<R>
where F: FnOnce(&mut FdTable) -> VFSResult<R> {
    let mut guard = SCHEDULER.lock();
    let sched = guard.as_mut().ok_or(VFSError::IOError)?;
    let task = sched.current().ok_or(VFSError::IOError)?;
    task.lock().with_fd_table(f)
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
        SCHEDULER.lock().as_ref().map(|sched| sched.current_idx())
    })
}

/// gets the current process name
pub fn name() -> String {
    interrupts::without_interrupts(|| {
        SCHEDULER
            .lock()
            .as_ref()
            .map(|sched| {
                sched.processes[sched.current_idx()].lock().name.clone()
            })
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
