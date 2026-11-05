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

use crate::system;
use crate::system::proc::{Process, ProcessLevel};
use crate::system::syscalls::SyscallFrame;

/// creates a new process by copying the current one, and returns the new pid.
pub fn fork(frame: &SyscallFrame) -> Result<u64, &'static str> {
    let current = system::proc::current().ok_or("no current process")?;
    let parent = current.lock();

    if parent.level != ProcessLevel::RING3 {
        return Err("fork is only supported for user processes");
    }

    let parent_as = parent
        .address_space
        .as_ref()
        .ok_or("user process has no address space")?;
    let child_as = parent_as.clone_user()?;

    let child = Process::new_forked(&parent, child_as, frame);
    let child_id = child.id;
    drop(parent);

    let mut sched_guard = system::proc::SCHEDULER.lock();
    let sched = sched_guard.as_mut().ok_or("scheduler not initialized")?;
    sched.add(child);

    Ok(child_id)
}
