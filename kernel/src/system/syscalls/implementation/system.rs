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

use crate::arch::syscalls::SyscallFrame;
use crate::system::posix::{KError, KResult};
use crate::system::syscalls::types::SyscallReturnCode;

pub fn uname(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let ptr = frame.rdi as *mut system::UtsName;
    if ptr.is_null() {
        return Err(KError::EINVAL);
    }
    log::trace!("uname syscall: ptr={:?}", ptr);

    // TODO: put this somewhere else
    fn make_cstr<const N: usize>(value: &str) -> [u8; N] {
        let mut buf = [0u8; N];

        let len = value.len().min(N.saturating_sub(1));
        buf[..len].copy_from_slice(&value.as_bytes()[..len]);

        buf
    }

    let utsname = system::UtsName {
        sysname: make_cstr("flower"),
        nodename: make_cstr("riria"),
        release: make_cstr(env!("CARGO_PKG_VERSION")),
        version: make_cstr(concat!(
            env!("CARGO_PKG_NAME"),
            " ",
            env!("CARGO_PKG_VERSION")
        )),
        machine: make_cstr("x86_64"),
        domainname: make_cstr("(none)"),
    };

    unsafe {
        core::ptr::write_volatile(ptr, utsname);
    }
    Ok(0)
}
