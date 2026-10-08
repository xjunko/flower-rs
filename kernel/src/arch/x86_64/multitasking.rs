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

use core::arch::naked_asm;

use crate::arch::{Arch, Multitasking, Processor};
use crate::proc;

#[repr(C)]
struct Frame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    ret: u64,
}

#[unsafe(naked)]
pub unsafe extern "C" fn _kernel_proc_trampoline() -> ! {
    naked_asm!(
        "mov rdi, r15",
        "call {wrapper}",
        "ud2",
        wrapper=sym __kernel_proc_entry
    );
}

#[allow(improper_ctypes_definitions)]
extern "C" fn __kernel_proc_entry(entry: fn()) -> ! {
    Processor::interrupts_enable();
    entry();
    proc::exit(0);
}

#[unsafe(naked)]
unsafe extern "C" fn switch_context(
    old_sp: *mut u64,
    new_sp: u64,
    new_root: u64,
) {
    naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp",
        "test rdx, rdx",
        "jz 2f",
        "mov cr3, rdx",
        "2:",
        "mov rsp, rsi",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
    );
}

impl Multitasking for Processor {
    fn prepare_stack(stack_top: u64, entry: fn()) -> u64 {
        let stack_ptr =
            (stack_top & !0xF) - core::mem::size_of::<Frame>() as u64;

        unsafe {
            (stack_ptr as *mut Frame).write(Frame {
                r15: entry as *const () as u64,
                r14: 0,
                r13: 0,
                r12: 0,
                rbx: 0,
                rbp: 0,
                ret: _kernel_proc_trampoline as *const () as u64,
            });
        }

        stack_ptr
    }

    fn switch_context(old_sp: *mut u64, new_sp: u64, new_root: u64) {
        unsafe {
            self::switch_context(old_sp, new_sp, new_root);
        }
    }
}

const _: () = assert!(core::mem::size_of::<Frame>() == 7 * 8);
const _: () = assert!(core::mem::offset_of!(Frame, r15) == 0);
const _: () = assert!(core::mem::offset_of!(Frame, ret) == 6 * 8);
