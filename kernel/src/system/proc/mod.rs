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

mod exit;
mod memory;
mod process;
mod scheduler;
mod sleep;

pub mod user;

use alloc::string::String;
use alloc::sync::Arc;

pub use memory::{KUserImage, KUserRange};
pub use process::*;
use spin::Mutex;

pub use self::exit::exit;
pub use self::sleep::sleep;
use crate::arch;
use crate::system::posix::errno::{KError, KResult};
use crate::system::proc::scheduler::Scheduler;
use crate::system::vfs::{FdKind, FdTable};

pub static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

/// schedules the process
pub fn schedule() {
    arch::interrupts::without(|| {
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
            Scheduler::switch_context(old_sp, new_sp, new_cr3);
        }
    });
}

/// spawns a new kernel process with the given entry point and name.
pub fn spawn(name: &str, entry: fn()) {
    let new_process = Process::new_kernel(name, entry);
    log::debug!("created process {}", new_process.name());
    arch::interrupts::without(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            log::debug!("adding process {} to scheduler", new_process.name());
            sched.add(new_process);
        }
    });
}

/// loops over the file descriptors of the current process
pub fn with_fd_table<F, R>(f: F) -> KResult<R>
where F: FnOnce(&mut FdTable) -> KResult<R> {
    let mut guard = SCHEDULER.lock();
    let sched = guard.as_mut().ok_or(KError::ESRCH)?;
    let task = sched.current().ok_or(KError::ESRCH)?;
    task.lock().with_fd_table(f)
}

/// returns a clone of the current process' fd entry.
pub fn fd(fd: usize) -> KResult<FdKind> {
    let current = current().ok_or(KError::ESRCH)?;
    current.lock().fds.get(fd).cloned()
}

/// returns the current process
pub fn current() -> Option<Arc<Mutex<Process>>> {
    arch::interrupts::without(|| {
        SCHEDULER.lock().as_mut().and_then(|sched| sched.current())
    })
}

/// returns the current pid
pub fn current_pid() -> Option<usize> {
    arch::interrupts::without(|| {
        SCHEDULER.lock().as_ref().map(|sched| sched.current_idx())
    })
}

/// gets the current process name
pub fn name() -> String {
    arch::interrupts::without(|| {
        SCHEDULER
            .lock()
            .as_ref()
            .map(|sched| {
                String::from(sched.processes[sched.current_idx()].lock().name())
            })
            .unwrap_or(String::from("undefined"))
    })
}

/// installs the scheduler, initializing the null process and adding it to the scheduler.
pub fn install() {
    let mut scheduler = Scheduler::new();
    scheduler.add(Process::null());
    arch::interrupts::without(|| {
        *SCHEDULER.lock() = Some(scheduler);
    });
}
