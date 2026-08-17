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

use flower_mono::prctl::ARCH_SET_FS;
use x86_64::VirtAddr;
use x86_64::registers::model_specific::FsBase;

use crate::memory::vmm::AddressSpace;
use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallError;

pub fn ctl(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let arg1 = frame.rdi;
    let arg2 = frame.rsi;

    match arg1 {
        ARCH_SET_FS => {
            if let fs_base = VirtAddr::new(arg2)
                && AddressSpace::current().is_mapped(fs_base)
            {
                log::debug!("writing FSBase with: {:#x}", fs_base);
                FsBase::write(fs_base);
                Ok(0)
            } else {
                Err(SyscallError::InvalidArgument)
            }
        },
        _ => Err(SyscallError::NoPermission),
    }
}
