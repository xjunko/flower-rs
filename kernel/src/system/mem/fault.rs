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

use crate::arch::addr::KVirtAddr;
use crate::arch::layout::PAGE_SIZE;
use crate::arch::x86_64::idt::print_stack_frame;
use crate::{println, system};

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
            system::proc::exit(1);
            return;
        },
    };

    // we might be able to handle this
    if let Some(current) = system::proc::current() {
        let proc = current.lock();

        // okay, it's from the user, probably fine.
        if let Some(address_space) = proc.address_space() {
            let page = KVirtAddr::new(fault_addr & !(PAGE_SIZE as u64 - 1));

            match address_space.demand_page(page) {
                Ok(true) => {
                    log::debug!(
                        "demand-paged user page at {:#x}",
                        page.as_u64()
                    );
                    return;
                },
                Ok(false) => {},
                Err(e) => {
                    log::error!(
                        "demand-page allocation FAILED at {:#x}: {}",
                        page.as_u64(),
                        e
                    );
                },
            }
        }
    }

    // shit hits the fan, panic out.
    log::error!("page fault triggered, in process: {}", system::proc::name());
    println!("CR2:        {:#x}", fault_addr);
    println!("error code: {:#x}", error_code);
    print_stack_frame(stack_frame);

    system::proc::exit(1);
}
