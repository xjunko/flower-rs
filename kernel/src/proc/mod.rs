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

use spin::Mutex;

use crate::arch::{Arch, Multitasking, Processor};
use crate::proc::process::{Process, ProcessState};
use crate::proc::scheduler::Scheduler;

mod process;
mod scheduler;
mod stub;

pub static SCHEDULER: Mutex<Option<Scheduler>> = Mutex::new(None);

pub fn schedule() {
    let interrupts_enabled = Processor::interrupts_enabled();
    if interrupts_enabled {
        Processor::interrupts_disable();
    }

    let context = {
        let mut guard = SCHEDULER.lock();
        if let Some(sched) = guard.as_mut() {
            sched.reap();
            sched.next_idx().map(|next| sched.switch_to(next))
        } else {
            panic!("scheduler not initialized");
        }
    };

    if let Some((old_sp, new_sp, new_stack_top, new_root)) = context {
        if new_stack_top != 0 {
            Processor::set_kernel_stack(new_stack_top);
        }
        Processor::switch_context(old_sp, new_sp, new_root);
    } else if interrupts_enabled {
        Processor::interrupts_enable();
    }
}

pub fn spawn(name: &str, entry: fn()) {
    let new_process = Process::new(name, entry);
    log::debug!("spawned process: {:?}", new_process.name.lock());

    Processor::interrupts_without(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            sched.add(new_process);
        }
    })
}

pub fn exit(status: u64) -> ! {
    Processor::interrupts_without(|| {
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                let proc = proc.lock();
                *proc.return_code.lock() = Some(status);
                *proc.state.lock() = ProcessState::Dead;
            } else {
                panic!("trying to exit while no process is running!");
            }
        } else {
            panic!("trying to exit while scheduler not initialized!");
        }
    });

    self::schedule();
    unreachable!()
}

pub fn install() {
    let mut scheduler = Scheduler::new();
    scheduler.add(stub::null_process());

    Processor::interrupts_without(|| {
        *SCHEDULER.lock() = Some(scheduler);
    })
}
