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

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use x86_64::PhysAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PhysFrame;

use crate::system::syscalls::SyscallFrame;
use crate::system::{self, vfs};

fn process_name_from_path(path: &str) -> String {
    path.rsplit('/').find(|part| !part.is_empty()).unwrap_or(path).to_string()
}

/// replaces the current process image with a new one from the given path.
pub fn execve(
    path: &str,
    argv: &[String],
    frame: &mut SyscallFrame,
) -> Result<(), &'static str> {
    let elf_data = vfs::__read(path)?;
    let name = process_name_from_path(path);

    let argv_storage: Vec<String> =
        if argv.is_empty() { alloc::vec![name.clone()] } else { argv.to_vec() };
    let argv_refs: Vec<&str> =
        argv_storage.iter().map(|arg| arg.as_str()).collect();

    let image = system::proc::user::build_user_image(&elf_data, &argv_refs)?;
    let new_cr3 = image.address_space.root_phys();

    let current = system::proc::current().ok_or("no current process")?;
    let mut proc = current.lock();

    if !proc.is_user() {
        return Err("execve is only supported for user processes");
    }

    let (current_frame, current_flags) = Cr3::read();
    if current_frame.start_address().as_u64() != new_cr3 {
        let new_frame = PhysFrame::containing_address(PhysAddr::new(new_cr3));
        unsafe {
            Cr3::write(new_frame, current_flags);
        }
    }

    let loaded = proc.load_image(image);
    proc.set_name(name);

    log::trace!(
        "execve: switching to '{}' at {:#x}",
        proc.name(),
        loaded.entry
    );
    unsafe {
        proc.switch_stack();
    }

    drop(loaded.previous_address_space);

    frame.rip = loaded.entry;
    frame.rsp = loaded.stack_pointer;
    frame.rax = 0;

    Ok(())
}
