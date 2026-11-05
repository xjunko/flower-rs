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

use flower_abi::time;

use crate::arch::addr::VirtAddr;
use crate::arch::syscalls::SyscallFrame;
use crate::system::posix::errno::{KError, KResult};
use crate::system::syscalls::types::SyscallReturnCode;
use crate::{arch, system};

pub fn clock_get_time(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let clock_id = frame.rdi as i32;
    let ptr = frame.rsi as *mut time::Timespec;

    log::trace!("clock_get_time: clock_id = {}, ptr = {:p}", clock_id, ptr);

    match clock_id {
        time::CLOCK_REALTIME => {
            // TODO: implement real time clock
            let timespec = time::Timespec { tv_sec: 0, tv_nsec: 0 };
            unsafe { ptr.write(timespec) };
            Ok(0)
        },
        time::CLOCK_MONOTONIC => {
            let now = crate::arch::timers::ns();
            let timespec = time::Timespec {
                tv_sec: (now / 1_000_000_000) as i64,
                tv_nsec: (now % 1_000_000_000) as i64,
            };
            unsafe { ptr.write(timespec) };
            Ok(0)
        },
        _ => Err(KError::ENOSYS),
    }
}

pub fn times(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let ptr = frame.rdi as *mut time::Times;
    log::trace!("times syscall: ptr={:?}", ptr);
    Err(KError::ENOSYS)
}

pub fn mtime(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let time_ptr = frame.rdi as *mut u64;
    if time_ptr.is_null() {
        return Err(KError::EINVAL);
    }

    arch::user::copy_to_user(
        VirtAddr::new(time_ptr.addr() as u64),
        arch::timers::ticks(),
    )
    .map_err(|_| KError::EFAULT)?;

    Ok(0)
}

pub fn msleep(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    // TODO: turn this into a nanosecond sleep from the syscall side.
    let ms = frame.rdi * 1_000_000;
    system::proc::sleep(ms);
    Ok(0)
}

pub fn nanosleep(frame: &mut SyscallFrame) -> KResult<SyscallReturnCode> {
    let req_ptr = frame.rdi as *const time::Timespec;
    if req_ptr.is_null() {
        return Err(KError::EINVAL);
    }
    log::trace!("nanosleep called with req_ptr: {:?}", req_ptr);

    let req = unsafe { &*req_ptr };
    let ns = req.tv_sec * 1_000_000_000 + req.tv_nsec;
    system::proc::sleep(ns as u64);
    Ok(0)
}
