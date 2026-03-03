use core::arch::naked_asm;

use x86_64::instructions::interrupts;

#[allow(improper_ctypes_definitions)]
extern "C" fn kernel_process_entry(entry: fn()) -> ! {
    interrupts::enable();
    entry();
    super::exit();
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
