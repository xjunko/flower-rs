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

use crate::arch;
use crate::system::proc::schedule;
use crate::system::{self};

/// exits the current process.
pub fn exit(status: u64) {
    arch::interrupts::without(|| {
        arch::stacks::write_context();

        if let Some(sched) = system::proc::SCHEDULER.lock().as_mut() {
            if let Some(proc) = sched.current() {
                proc.lock().mark_exited(status);
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
