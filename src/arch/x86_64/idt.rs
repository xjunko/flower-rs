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

use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::arch::x86_64::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::arch::x86_64::interrupts::{
    InterruptIndex, spurious_interrupt_handler, timer_interrupt_handler,
};
use crate::devices::ps2::keyboard;
use crate::{memory, println};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.general_protection_fault.set_handler_fn(gpf_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available.set_handler_fn(device_not_available_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(memory::fault::page_fault_handler);

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    // ps2/keyboard
    idt[InterruptIndex::Keyboard.as_u8()]
        .set_handler_fn(keyboard::keyboard_interrupt_handler);

    // spurious
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Spurious.as_u8()]
        .set_handler_fn(spurious_interrupt_handler);

    idt
});

pub fn install() {
    IDT.load();
    log::info!("IDT loaded.");
}

pub fn print_stack_frame(frame: InterruptStackFrame) {
    println!("RIP:    {:#x}", frame.instruction_pointer.as_u64());
    println!("CS:     {:#x}", frame.code_segment.0);
    println!("RFLAGS: {:#x}", frame.cpu_flags);
    println!("RSP:    {:#x}", frame.stack_pointer);
    println!("SS:     {:#x}", frame.stack_segment.0);
}

extern "x86-interrupt" fn gpf_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("general Protection Fault triggered!");
    println!("error code: {:#x}", error_code);
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame,
) {
    log::error!("invalid opcode (#UD) triggered!");
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn device_not_available_handler(
    stack_frame: InterruptStackFrame,
) {
    log::error!("device not available (#NM) triggered!");
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    log::error!("double fault triggered!");
    print_stack_frame(stack_frame);
    panic!("");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::warn!("breakpoint triggered!");
    print_stack_frame(stack_frame);
}
