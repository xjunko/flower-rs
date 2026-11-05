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
use core::ffi::{CStr, c_char};

use crate::system::posix::errno::{KError, KResult};
use crate::system::syscalls::types::{SyscallFrame, SyscallReturnCode};
use crate::system::{self};

pub fn exit(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    system::proc::exit(frame.rdi);
    unreachable!();
}

pub fn exit_group(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    // TODO: we dont support multithreading yet, so this is fine
    system::proc::exit(frame.rdi);
    unreachable!();
}

pub fn fork(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    system::proc::user::fork(frame).map_err(|e| {
        log::error!("fork failed: {}", e);
        KError::EINVAL
    })
}

pub fn waitpid(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    system::proc::user::waitpid(frame.rdi).map_err(|e| {
        if e == "no child process" {
            KError::ECHILD
        } else {
            log::error!("waitpid failed: {}", e);
            KError::EINVAL
        }
    })
}

pub fn wait4(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    system::proc::user::waitpid(frame.rdi).map_err(|e| {
        if e == "no child process" {
            KError::ECHILD
        } else {
            log::error!("wait4 failed: {}", e);
            KError::EINVAL
        }
    })
}

pub fn execve(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    const MAX_ARGV: usize = 32;

    let path_ptr = frame.rdi as *const c_char;
    if path_ptr.is_null() {
        return Err(KError::EINVAL);
    }

    let path = unsafe { CStr::from_ptr(path_ptr) }
        .to_str()
        .map_err(|_| KError::EINVAL)?;

    let argv_ptr = frame.rsi as *const *const c_char;
    let mut argv = Vec::<String>::new();

    if !argv_ptr.is_null() {
        for idx in 0..MAX_ARGV {
            let arg_ptr = unsafe { *argv_ptr.add(idx) };
            if arg_ptr.is_null() {
                break;
            }

            let arg = unsafe { CStr::from_ptr(arg_ptr) }
                .to_str()
                .map_err(|_| KError::EINVAL)?;
            argv.push(arg.to_string());
        }
    }

    if argv.is_empty() {
        argv.push(path.to_string());
    }

    if let Err(reason) = system::proc::user::execve(path, &argv, frame) {
        log::error!("execve failed for path '{}': {:?}", path, reason);
        return Err(KError::ENOENT);
    }
    Ok(0)
}
