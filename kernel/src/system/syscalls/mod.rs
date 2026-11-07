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

mod implementation;
mod types;

pub use types::SyscallFrame;
use x86_64::instructions::interrupts;

use crate::system::posix::errno::KError;
use crate::system::syscalls::implementation::SYSCALL_HANDLERS;
use crate::system::syscalls::types::SyscallReturnCode;

#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(frame: *mut SyscallFrame) {
    interrupts::without_interrupts(|| {
        let frame = unsafe { &mut *frame };
        let result = syscall_handler_unwrapped(frame.rax as u64, frame);
        frame.rax = result as isize;
    });
}

fn syscall_handler_unwrapped(
    num: u64,
    frame: &mut SyscallFrame,
) -> SyscallReturnCode {
    if let Some(handler) = SYSCALL_HANDLERS.get(num as usize).and_then(|h| *h) {
        match handler(frame) {
            Ok(result) => result,
            Err(e) => {
                log::error!("syscall {} failed with error: {:?}", num, e);
                -(e as i32) as SyscallReturnCode
            },
        }
    } else {
        log::debug!("unimplemented syscall {} | frame: {:?}", num, frame);
        -(KError::ENOSYS as i32) as SyscallReturnCode
    }
}
