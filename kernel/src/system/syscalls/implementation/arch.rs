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

use flower_abi::arch;
use x86_64::VirtAddr as x86VirtAddr;
use x86_64::registers::model_specific::FsBase;

use crate::arch::addr::VirtAddr;
use crate::system::mem::vmm::AddressSpace;
use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallError;

pub fn prctl(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let address_space = AddressSpace::current();

    let arg1 = frame.rdi;
    let arg2 = frame.rsi;

    log::trace!("prctl called with arg1: {:#x}, arg2: {:#x}", arg1, arg2);

    match arg1 {
        arch::ARCH_SET_FS => {
            if let fs_base = VirtAddr::new(arg2)
                && address_space.is_mapped(fs_base)
            {
                log::debug!("writing FSBase with: {:#x}", fs_base);
                FsBase::write(x86VirtAddr::new(fs_base.as_u64()));
                Ok(0)
            } else {
                Err(SyscallError::InvalidArgument)
            }
        },
        _ => Err(SyscallError::NoPermission),
    }
}
