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

use crate::arch::x86_64::layout::{PAGE_SIZE, USER_STACK_TOP_PAGE};
use crate::system::proc::user::stack::build_user_image;
use crate::system::proc::{Process, SCHEDULER};

mod execve;
mod fork;
mod stack;
mod waitpid;

pub use execve::execve;
pub use fork::fork;
pub use waitpid::waitpid;

/// spawns an elf process with the given name and elf bytes.
pub fn spawn_elf(name: &str, elf_data: &[u8]) -> Result<u64, &'static str> {
    let argv = [name];
    let image = build_user_image(elf_data, &argv)?;

    let mut proc = Process::new_user(
        name,
        image.address_space,
        image.entry,
        image.stack_ptr,
        image.heap_start + PAGE_SIZE as u64,
    );

    // Set the stack/heap bounds after process creation
    proc.set_user_stack_bounds(image.stack_bottom, USER_STACK_TOP_PAGE);
    proc.set_user_heap_bounds(
        image.heap_start + PAGE_SIZE as u64,
        image.heap_max,
    );

    let proc_id = proc.id;
    log::trace!(
        "created process {} with entry point {:#x}",
        proc.name,
        image.entry
    );

    proc.set_user_stack_bounds(image.stack_bottom, USER_STACK_TOP_PAGE);
    proc.set_user_heap_bounds(
        image.heap_start + PAGE_SIZE as u64,
        image.heap_max,
    );

    if let Some(sched) = SCHEDULER.lock().as_mut() {
        sched.add(proc);
    }

    Ok(proc_id)
}
