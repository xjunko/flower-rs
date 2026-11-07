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

use core::ffi::c_int;

use flower_abi::memory::{
    MAP_ANONYMOUS, MAP_FIXED, MAP_PRIVATE, MAP_SHARED, PROT_EXEC, PROT_NONE,
    PROT_WRITE,
};

use crate::arch::addr::{KVirtAddr, align_up};
use crate::arch::generic::paging::MapFlags;
use crate::arch::layout::PAGE_SIZE;
use crate::system;
use crate::system::posix::errno::{KError, KResult};
use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallReturnCode;
use crate::system::vfs::FdKind;

const PAGE_SIZE_U64: u64 = PAGE_SIZE as u64;

fn align_range_up(value: u64) -> KResult<u64> {
    value
        .checked_add(PAGE_SIZE_U64 - 1)
        .map(|value| value & !(PAGE_SIZE_U64 - 1))
        .ok_or(KError::EINVAL)
}

fn prot_to_flags(prot: u64) -> MapFlags {
    let mut flags = MapFlags::empty();

    if prot != PROT_NONE {
        flags |= MapFlags::USER;
    }

    if prot & PROT_WRITE != 0 {
        flags |= MapFlags::WRITE;
    }

    if prot & PROT_EXEC != 0 {
        flags |= MapFlags::EXEC;
    }

    flags
}

pub fn brk(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let requested = frame.rdi;
    let current = system::proc::current().ok_or(KError::EINVAL)?;
    let mut proc = current.lock();

    let heap = proc.heap().ok_or(KError::EINVAL)?;
    let current_break = heap.brk();

    if requested == 0 || requested == current_break {
        return Ok(current_break);
    }

    if requested < heap.base() || requested > heap.limit() {
        return Ok(current_break);
    }

    let flags = MapFlags::READ | MapFlags::WRITE | MapFlags::USER;

    if requested > current_break {
        let first = align_up(current_break, PAGE_SIZE_U64);
        let last = align_up(requested, PAGE_SIZE_U64);

        if first < last {
            proc.address_space()
                .ok_or(KError::EINVAL)?
                .ensure_user_pages(first, last, flags)
                .map_err(|_| KError::ENOMEM)?;
        }
    } else {
        // release the pages above the new break. the heap vma still covers the
        // whole range, so a later growth can fault them back in.
        let first = align_up(requested, PAGE_SIZE_U64);
        let last = align_up(current_break, PAGE_SIZE_U64);

        if first < last {
            let address_space =
                proc.address_space_mut().ok_or(KError::EINVAL)?;

            let mut page = first;
            while page < last {
                let virt = KVirtAddr::new(page);
                if address_space.is_mapped(virt) {
                    let _ = address_space.unmap_page(virt);
                }
                page += PAGE_SIZE_U64;
            }
        }
    }

    if let Some(heap) = proc.heap_mut() {
        heap.set_brk(requested);
    }

    Ok(requested)
}

pub fn mmap(frame: &mut SyscallFrame) -> KResult<u64> {
    let addr = frame.rdi;
    let size = frame.rsi;
    let prot = frame.rdx;
    let flags = frame.r10;
    let fd = frame.r8 as i64;
    let offset = frame.r9;

    log::debug!(
        "mmap: fd={}, offset={}, flags={}, size={}",
        fd,
        offset,
        flags,
        size
    );

    if size == 0 {
        log::error!("mmap failed: size is 0");
        return Err(KError::EINVAL);
    }

    let fixed = flags & MAP_FIXED != 0;

    if fixed && (addr == 0 || !addr.is_multiple_of(PAGE_SIZE_U64)) {
        log::error!("mmap failed: MAP_FIXED address is not page-aligned");
        return Err(KError::EINVAL);
    }

    if !offset.is_multiple_of(PAGE_SIZE_U64) {
        log::error!("mmap failed: offset is not page-aligned");
        return Err(KError::EINVAL);
    }

    let len = align_range_up(size)?;
    let prot_flags = prot_to_flags(prot);

    if fd != -1 && flags & MAP_ANONYMOUS == 0 {
        let has_shared = flags & MAP_SHARED != 0;
        let has_private = flags & MAP_PRIVATE != 0;

        if has_shared == has_private {
            log::error!(
                "mmap failed: file mapping needs exactly one of MAP_SHARED/MAP_PRIVATE"
            );
            return Err(KError::EINVAL);
        }
    }

    let current = system::proc::current().ok_or(KError::EINVAL)?;
    let mut proc = current.lock();

    // ask the file for its backing frames before borrowing the address space.
    let file_phys = if fd != -1 && flags & MAP_ANONYMOUS == 0 {
        Some(proc.with_fd_table(|table| match table.get(fd as usize)? {
            FdKind::File(file) => {
                file.mmap(len as usize, prot as c_int, flags as c_int, offset)
            },
            _ => Err(KError::ENOSYS),
        })?)
    } else {
        None
    };

    let address_space = proc.address_space_mut().ok_or(KError::EINVAL)?;

    // MAP_FIXED uses the address verbatim. hints are honoured when the range
    // is actually free, otherwise they are ignored like linux does.
    let start = if fixed {
        addr
    } else if addr != 0 {
        let hint = addr & !(PAGE_SIZE_U64 - 1);
        if address_space.user_range_free(hint, len as usize) {
            hint
        } else {
            address_space
                .find_free_user_range(len as usize)
                .ok_or(KError::ENOMEM)?
        }
    } else {
        address_space
            .find_free_user_range(len as usize)
            .ok_or(KError::ENOMEM)?
    };

    let end = start.checked_add(len).ok_or(KError::EINVAL)?;

    if fixed {
        // replace whatever used to live there.
        address_space
            .unmap_user_range(start, end)
            .map_err(|_| KError::EINVAL)?;
    }

    let result = match file_phys {
        Some(phys) => {
            address_space.map_user_file(start, len as usize, prot_flags, phys)
        },
        None if flags & MAP_SHARED != 0 => {
            address_space.map_user_shared(start, len as usize, prot_flags)
        },
        None => {
            address_space.map_user_anonymous(start, len as usize, prot_flags)
        },
    };

    result.map_err(|e| {
        log::error!(
            "mmap failed: addr={:#x}, flags={:#x}, fixed={}, start={:#x}, size={:#x}: {}",
            addr,
            flags,
            fixed,
            start,
            len,
            e
        );
        KError::ENOMEM
    })?;

    log::debug!("mmap: mapped {:#x}-{:#x}", start, end);
    Ok(start)
}

pub fn munmap(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let addr = frame.rdi;
    let size = frame.rsi;

    if size == 0 {
        return Err(KError::EINVAL);
    }

    let end = align_range_up(addr.checked_add(size).ok_or(KError::EINVAL)?)?;
    let start = addr & !(PAGE_SIZE_U64 - 1);

    log::debug!("munmap: {:#x}-{:#x}", start, end);

    let current = system::proc::current().ok_or(KError::EINVAL)?;
    let mut proc = current.lock();

    proc.address_space_mut()
        .ok_or(KError::EINVAL)?
        .unmap_user_range(start, end)
        .map_err(|_| KError::EINVAL)?;

    Ok(0)
}

pub fn mprotect(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let addr = frame.rdi;
    let size = frame.rsi;
    let prot = frame.rdx;

    if size == 0 || !addr.is_multiple_of(PAGE_SIZE_U64) {
        return Err(KError::EINVAL);
    }

    let end = align_range_up(addr.checked_add(size).ok_or(KError::EINVAL)?)?;
    let flags = prot_to_flags(prot);

    let current = system::proc::current().ok_or(KError::EINVAL)?;
    let mut proc = current.lock();

    proc.address_space_mut()
        .ok_or(KError::EINVAL)?
        .protect_user_range(addr, end, flags)
        .map_err(|_| KError::EINVAL)?;

    Ok(0)
}
