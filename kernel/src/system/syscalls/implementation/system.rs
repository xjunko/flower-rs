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

use flower_abi::system;

use crate::arch;
use crate::arch::addr::VirtAddr;
use crate::arch::syscalls::SyscallFrame;
use crate::system::posix::errno::{KError, KResult};
use crate::system::posix::utsname::UTSNAME;
use crate::system::syscalls::types::SyscallReturnCode;

pub fn uname(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let ptr = frame.rdi as *mut system::UtsName;
    if ptr.is_null() {
        return Err(KError::EINVAL);
    }
    log::trace!("uname syscall: ptr={:?}", ptr);

    let utsname = UTSNAME.lock();
    arch::user::copy_to_user(VirtAddr::new(ptr.addr() as u64), utsname.clone())
        .map_err(|_| KError::EFAULT)?;

    Ok(0)
}
