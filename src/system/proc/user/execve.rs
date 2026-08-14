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

use crate::arch::x86_64::layout::{PAGE_SIZE, USER_STACK_TOP_PAGE};
use crate::system;
use crate::system::proc::ProcessLevel;
use crate::system::syscalls::SyscallFrame;
use crate::system::vfs2::file::OpenFlags;
use crate::system::vfs2::perm::Credentials;

fn process_name_from_path(path: &str) -> String {
    path.rsplit('/').find(|part| !part.is_empty()).unwrap_or(path).to_string()
}

/// replaces the current process image with a new one from the given path.
pub fn execve(
    path: &str,
    argv: &[String],
    frame: &mut SyscallFrame,
) -> Result<(), &'static str> {
    let elf_data;

    if let Ok(elf_file) =
        system::vfs2::open(path, OpenFlags::RDONLY, Credentials::ROOT)
    {
        let metadata =
            elf_file.metadata().map_err(|_| "failed to get metadata")?;
        let mut buffer = alloc::vec![0u8; metadata.size];
        elf_file
            .read(buffer.as_mut_slice())
            .map_err(|_| "failed to read file")?;
        elf_data = buffer;
    } else {
        return Err("failed to open file");
    }

    let name = process_name_from_path(path);

    let argv_storage: Vec<String> =
        if argv.is_empty() { alloc::vec![name.clone()] } else { argv.to_vec() };
    let argv_refs: Vec<&str> =
        argv_storage.iter().map(|arg| arg.as_str()).collect();

    let image = system::proc::user::build_user_image(
        elf_data.as_slice(),
        argv_refs.as_slice(),
    )?;
    let new_cr3 = image.address_space.cr3();
    let (current_frame, current_flags) = Cr3::read();
    if current_frame.start_address().as_u64() != new_cr3 {
        let new_frame = PhysFrame::containing_address(PhysAddr::new(new_cr3));
        unsafe {
            Cr3::write(new_frame, current_flags);
        }
    }

    let current = system::proc::current().ok_or("no current process")?;
    let mut proc = current.lock();

    if proc.level != ProcessLevel::RING3 {
        return Err("execve is only supported for user processes");
    }

    let old_address_space = proc.address_space.take();

    proc.name = name;
    proc.address_space = Some(image.address_space);
    proc.cr3 = new_cr3;
    proc.set_user_entry(image.entry);
    proc.set_user_stack(image.stack_ptr);
    proc.set_user_heap(image.heap_start);

    proc.set_user_heap_position(image.heap_start + PAGE_SIZE as u64);

    proc.set_user_heap_bounds(
        image.heap_start + PAGE_SIZE as u64,
        image.heap_max,
    );

    proc.set_user_stack_bounds(
        image.stack_bottom,
        USER_STACK_TOP_PAGE + PAGE_SIZE as u64,
    );

    proc._fsbase = 0;

    log::trace!("execve: address space switched, preparing to switch stacks");
    unsafe {
        proc.switch_stack();
    }
    log::trace!(
        "execve: stacks switched, dropping old address space and writing fsbase"
    );

    drop(old_address_space);

    frame.rip = image.entry;
    frame.rsp = image.stack_ptr;
    frame.rax = 0;

    Ok(())
}
