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

use x86_64::VirtAddr;
use x86_64::registers::control::{Efer, EferFlags};
use x86_64::registers::model_specific::{KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use crate::arch;
use crate::arch::x86_64::gdt;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: isize,

    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[repr(C, align(16))]
struct SyscallContext {
    user: u64,
    kernel: u64,
}

static mut SYSCALL_CONTEXT: SyscallContext =
    SyscallContext { user: 0, kernel: 0 };

#[allow(static_mut_refs)]
pub fn set_kernel_stack(stack_top: u64) {
    unsafe {
        SYSCALL_CONTEXT.kernel = stack_top;
    }
}

#[allow(static_mut_refs)]
pub fn set_user_stack(stack_top: u64) {
    unsafe {
        SYSCALL_CONTEXT.user = stack_top;
    }
}

#[allow(static_mut_refs)]
pub fn write_cpu_context() {
    unsafe {
        let cpu_local = &SYSCALL_CONTEXT as *const _ as u64;
        KernelGsBase::write(VirtAddr::new(cpu_local));
    }
}

#[allow(static_mut_refs)]
pub fn install() {
    arch::interrupts::without(|| {
        let segments = gdt::segments();

        unsafe {
            // enable syscall
            let efer = Efer::read();
            Efer::write(efer | EferFlags::SYSTEM_CALL_EXTENSIONS);

            // disable interrupts during syscall
            SFMask::write(RFlags::INTERRUPT_FLAG);

            // set segments
            Star::write(
                segments.user_code,
                segments.user_data,
                segments.kernel_code,
                segments.kernel_data,
            )
            .expect("failed to write STAR");

            // entry point
            LStar::write(VirtAddr::new(syscall_entry as *const () as u64));

            let cpu_local = &SYSCALL_CONTEXT as *const _ as u64;
            KernelGsBase::write(VirtAddr::new(cpu_local));
        }
    })
}

#[unsafe(naked)]
unsafe extern "C" fn syscall_entry() {
    naked_asm!(
        "swapgs",
        // save user rsp to gs:[0], and load kernel rsp from gs:[8]
        "mov gs:[0], rsp",
        "mov rsp, gs:[8]",
        // top half of iret frame (ss, rsp, rflags, cs, rip)
        "push {user_ss}",
        "push gs:[0]",
        "push r11",
        "push {user_cs}",
        "push rcx",
        // bottom half
        "push rax",
        "push rcx",
        "push rdx",
        "push rbx",
        "push rbp",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        //
        "mov rdi, rsp",
        //
        "call {handler}",
        //
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rbp",
        "pop rbx",
        "pop rdx",
        "pop rcx",
        "pop rax",
        // done
        "swapgs",
        "iretq",
        //
        handler = sym crate::system::syscalls::syscall_handler,
        user_ss = const 0x1b, // hardcoded
        user_cs = const 0x23, // hardcoded
    )
}
