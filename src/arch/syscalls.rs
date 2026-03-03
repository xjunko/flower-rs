use x86_64::structures::idt::InterruptStackFrame;

use crate::error;

pub extern "x86-interrupt" fn syscall_handler(_stack_frame: InterruptStackFrame) {
    error!("syscall triggered!");
    super::apic::eoi();
}
