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

use crate::arch::addr::VirtAddr;
use crate::arch::x86_64::{gdt, syscalls};
use crate::arch::{Multitasking, Processor};

impl Multitasking for Processor {
    fn prepare_stack_kernel(_kernel_stack_top: u64, _entry: fn()) -> u64 {
        todo!()
    }

    fn prepare_stack_user(
        _kernel_stack_top: u64,
        _entry: fn(),
        _user_stack_top: u64,
    ) -> u64 {
        todo!()
    }

    fn stack_set_kernel(addr: VirtAddr) {
        gdt::set_kernel_stack(addr.as_u64());
        syscalls::set_kernel_stack(addr.as_u64());
    }

    fn stack_set_user(addr: VirtAddr) {
        syscalls::set_user_stack(addr.as_u64());
    }

    fn stack_write_context() { syscalls::write_cpu_context(); }

    fn switch_context(
        _old_sp: *mut u64,
        _new_sp: u64,
        _new_root: Option<<Self as crate::arch::Paging>::Root>,
    ) {
        todo!()
    }
}
