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

use x86_64::instructions::interrupts;

use crate::arch;
use crate::system::proc::{ProcessState, SCHEDULER, schedule};
use crate::system::{self};

/// sleeps the current process for the given number of nanoseconds.
pub fn sleep(nanos: u64) {
    let wake_at = arch::x86_64::timer::get_ns() + nanos;

    interrupts::without_interrupts(|| {
        system::syscalls::write_cpu_context();
        if let Some(sched) = SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                let mut proc = proc.lock();
                proc.state = ProcessState::Sleeping(wake_at);
            } else {
                panic!("trying to sleep while no process is running!");
            }
        } else {
            panic!("trying to sleep while not initialized!");
        }
    });
    schedule();
}
