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

use core::mem::size_of;

use crate::arch::addr::VirtAddr;
use crate::arch::generic::paging::MapFlags;
use crate::arch::{Paging, Processor, Userspace};
use crate::system::posix::errno::{KError, KResult};

// TODO: move this?
const USER_ADDR_END: u64 = 0x0000_7FFF_FFFF_FFFF;

fn validate_user_range(addr: VirtAddr, len: usize, write: bool) -> KResult<()> {
    if len == 0 {
        return Ok(());
    }

    let start = addr.as_u64();
    let end = start.checked_add(len as u64).ok_or(KError::EFAULT)?;

    if start > USER_ADDR_END || end > USER_ADDR_END + 1 {
        return Err(KError::EFAULT);
    }

    let root = <Processor as Paging>::paging_root_active();
    let page_size = <Processor as Paging>::PAGE_SIZE as u64;
    let mut page = start & !(page_size - 1);

    while page < end {
        let flags =
            <Processor as Paging>::paging_translate(root, VirtAddr::new(page))
                .map(|(_, flags)| flags)
                .ok_or(KError::EFAULT)?;

        if !flags.contains(MapFlags::USER)
            || (write && !flags.contains(MapFlags::WRITE))
        {
            return Err(KError::EFAULT);
        }

        page = page.checked_add(page_size).ok_or(KError::EFAULT)?;
    }

    Ok(())
}

impl Userspace for Processor {
    fn copy_to_user<T: Copy>(src: VirtAddr, data: T) -> KResult<()> {
        validate_user_range(src, size_of::<T>(), true)?;
        unsafe {
            core::ptr::write_unaligned(src.as_mut_ptr::<T>(), data);
        }

        Ok(())
    }

    fn copy_from_user<T: Copy>(dest: VirtAddr) -> KResult<T> {
        validate_user_range(dest, size_of::<T>(), false)?;
        Ok(unsafe { core::ptr::read_unaligned(dest.as_ptr::<T>()) })
    }
}
