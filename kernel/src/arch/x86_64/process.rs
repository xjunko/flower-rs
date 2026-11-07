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

use core::arch::{asm, naked_asm};

use crate::arch::addr::KVirtAddr;
use crate::arch::x86_64::{gdt, syscalls};
use crate::arch::{self, Multitasking, Processor};
use crate::system::proc;

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

#[allow(improper_ctypes_definitions)]
extern "C" fn _kernel_proc_entry(entry: fn()) -> ! {
    arch::interrupts::enable();
    entry();
    proc::exit(0);
    unreachable!();
}

#[unsafe(naked)]
unsafe extern "C" fn __kernel_proc_trampoline() -> ! {
    naked_asm!(
        "mov rdi, r15",
        "call {wrapper}",
        "ud2",
        wrapper=sym _kernel_proc_entry
    );
}

#[allow(improper_ctypes_definitions)]
extern "C" fn _user_proc_entry(user_entry: u64, user_stack: u64) -> ! {
    arch::interrupts::enable();
    {
        let segments = gdt::segments();

        let user_cs = segments.user_code.0 as u64;
        let user_ss = segments.user_data.0 as u64;

        unsafe {
            asm!(
                "push {user_ss}",
                "push {user_stack}",
                "push 0x202",
                "push {user_cs}",
                "push {user_entry}",
                "iretq",
                user_ss = in(reg) user_ss,
                user_stack = in(reg) user_stack,
                user_cs = in(reg) user_cs,
                user_entry = in(reg) user_entry,
                options(noreturn)
            )
        }
    }
}

#[unsafe(naked)]
unsafe extern "C" fn __user_proc_trampoline() -> ! {
    naked_asm!(
        "mov rdi, r15",
        "mov rsi, r14",
        "call {wrapper}",
        "ud2",
        wrapper=sym _user_proc_entry
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn __fork_proc_trampoline() -> ! {
    naked_asm!(
        "pop r15", "pop r14", "pop r13", "pop r12", "pop r11", "pop r10",
        "pop r9", "pop r8", "pop rdi", "pop rsi", "pop rbp", "pop rbx",
        "pop rdx", "pop rcx", "pop rax", "iretq",
    );
}

#[unsafe(naked)]
unsafe extern "C" fn switch_context(
    old_sp: *mut u64,
    new_sp: u64,
    new_cr3: u64,
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
    fn prepare_stack_kernel(kernel_stack_top: u64, entry: fn()) -> u64 {
        let stack_ptr =
            (kernel_stack_top & !0xF) - core::mem::size_of::<Frame>() as u64;

        unsafe {
            (stack_ptr as *mut Frame).write(Frame {
                r15: entry as *const () as u64,
                r14: 0,
                r13: 0,
                r12: 0,
                rbx: 0,
                rbp: 0,
                ret: __kernel_proc_trampoline as *const () as u64,
            });
        }

        stack_ptr
    }

    fn prepare_stack_user(
        kernel_stack_top: u64,
        entry: u64,
        user_stack_top: u64,
    ) -> u64 {
        let stack_ptr =
            (kernel_stack_top & !0xF) - core::mem::size_of::<Frame>() as u64;

        unsafe {
            (stack_ptr as *mut Frame).write(Frame {
                r15: entry as *const () as u64,
                r14: user_stack_top,
                r13: 0,
                r12: 0,
                rbx: 0,
                rbp: 0,
                ret: __user_proc_trampoline as *const () as u64,
            });
        }

        stack_ptr
    }

    fn prepare_stack_fork(kernel_stack_top: u64) -> u64 {
        let stack_ptr =
            (kernel_stack_top & !0xF) - core::mem::size_of::<Frame>() as u64;

        unsafe {
            (stack_ptr as *mut Frame).write(Frame {
                r15: 0,
                r14: 0,
                r13: 0,
                r12: 0,
                rbx: 0,
                rbp: 0,
                ret: __fork_proc_trampoline as *const () as u64,
            });
        }

        stack_ptr
    }

    fn stack_set_kernel(addr: KVirtAddr) {
        gdt::set_kernel_stack(addr.as_u64());
        syscalls::set_kernel_stack(addr.as_u64());
    }

    fn stack_set_user(addr: KVirtAddr) {
        syscalls::set_user_stack(addr.as_u64());
    }

    fn stack_write_context() { syscalls::write_cpu_context(); }

    fn switch_context(
        old_sp: *mut u64,
        new_sp: u64,
        new_root: Option<<Self as crate::arch::Paging>::Root>,
    ) {
        let new_cr3 = new_root.unwrap_or(0);
        unsafe { self::switch_context(old_sp, new_sp, new_cr3) }
    }
}
