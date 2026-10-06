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

use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

use crate::println;

/// TODO: remove this!!
fn print_stack_frame(frame: InterruptStackFrame) {
    println!("RIP:    {:#x}", frame.instruction_pointer.as_u64());
    println!("CS:     {:#x}", frame.code_segment.0);
    println!("RFLAGS: {:#x}", frame.cpu_flags);
    println!("RSP:    {:#x}", frame.stack_pointer);
    println!("SS:     {:#x}", frame.stack_segment.0);
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_addr = match Cr2::read() {
        Ok(addr) => addr.as_u64(),
        Err(addr_err) => {
            log::error!(
                "page fault triggered, but CR2 is invalid: {:?}",
                addr_err
            );
            return;
        },
    };

    log::error!("page fault triggered");
    println!("cr2:        {:#x}", fault_addr);
    println!("error code: {:#x}", error_code);
    print_stack_frame(stack_frame);
}
