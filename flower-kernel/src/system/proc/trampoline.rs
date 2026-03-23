use core::arch::{asm, naked_asm};

use x86_64::instructions::interrupts;

use crate::arch::gdt;
use crate::system::proc;

#[allow(improper_ctypes_definitions)]
extern "C" fn kernel_process_entry(entry: fn()) -> ! {
    interrupts::enable();
    entry();
    proc::exit();
    unreachable!();
}

#[unsafe(naked)]
pub unsafe extern "C" fn kernel_trampoline_entry() -> ! {
    naked_asm!(
        "mov rdi, r15",
        "call {wrapper}",
        "ud2",
        wrapper=sym kernel_process_entry
    );
}

#[allow(improper_ctypes_definitions)]
extern "C" fn user_process_entry(user_entry: u64, user_stack: u64) -> ! {
    interrupts::enable();
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
pub unsafe extern "C" fn user_trampoline_entry() -> ! {
    naked_asm!(
        "mov rdi, r15",
        "mov rsi, r14",
        "call {wrapper}",
        "ud2",
        wrapper=sym user_process_entry
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn fork_return_trampoline() -> ! {
    naked_asm!(
        "pop r15", "pop r14", "pop r13", "pop r12", "pop r11", "pop r10",
        "pop r9", "pop r8", "pop rdi", "pop rsi", "pop rbp", "pop rbx",
        "pop rdx", "pop rcx", "pop rax", "swapgs", "iretq",
    );
}
