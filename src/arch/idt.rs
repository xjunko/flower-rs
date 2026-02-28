use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{
    arch::{
        gdt::DOUBLE_FAULT_IST_INDEX,
        interrupts::{InterruptIndex, spurious_interrupt_handler, timer_interrupt_handler},
    },
    error, println, warn,
};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.general_protection_fault.set_handler_fn(gpf_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available
        .set_handler_fn(device_not_available_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    // spurious
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Spurious.as_u8()].set_handler_fn(spurious_interrupt_handler);

    idt
});

pub fn install() {
    IDT.load();
}

pub fn print_stack_frame(frame: InterruptStackFrame) {
    println!("RIP:    {:#x}", frame.instruction_pointer.as_u64());
    println!("CS:     {:#x}", frame.code_segment.0);
    println!("RFLAGS: {:#x}", frame.cpu_flags);
    println!("RSP:    {:#x}", frame.stack_pointer);
    println!("SS:     {:#x}", frame.stack_segment.0);
}

extern "x86-interrupt" fn gpf_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("general Protection Fault triggered!");
    println!("error code: {:#x}", error_code);
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    error!("invalid opcode (#UD) triggered!");
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    error!("device not available (#NM) triggered!");
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    error!("double fault triggered!");
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    warn!("breakpoint triggered!");
    print_stack_frame(stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    error!("page fault triggered!");
    println!("error code: {:#x}", error_code);
    print_stack_frame(stack_frame);
}
