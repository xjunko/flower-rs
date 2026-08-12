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

use limine::mp::Cpu;

use crate::arch;
use crate::boot::limine::SMP_REQUEST;

unsafe extern "C" fn __smp_entry(ap: &Cpu) -> ! {
    log::info!("SMP: core {} started.", ap.lapic_id);
    arch::x86_64::interrupts::disable();
    arch::x86_64::halt();
}

pub fn install() {
    if let Some(smp) = SMP_REQUEST.get_response() {
        let cpus = smp.cpus();

        log::info!(
            "SMP: found {} cores, BSP is {}.",
            cpus.len(),
            smp.bsp_lapic_id()
        );

        for cpu in cpus {
            if cpu.lapic_id == smp.bsp_lapic_id() {
                continue; // dont want to mess with this one
            }
            cpu.goto_address.write(__smp_entry);
        }
    } else {
        log::error!("SMP: not supported, not good.");
    }
}
