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

pub mod apic;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod layout;

use core::arch::asm;

use raw_cpuid::CpuId;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

pub fn install_cpu_features() {
    let cpuid = CpuId::new();
    if let Some(finfo) = cpuid.get_feature_info() {
        assert!(finfo.has_fxsave_fxstor(), "FXSAVE/FXSTOR not supported");
        assert!(finfo.has_mmx(), "MMX not supported");
        assert!(finfo.has_sse(), "SSE not supported");
        assert!(finfo.has_fpu(), "FPU not supported");

        unsafe {
            Cr0::update(|flags| {
                flags.remove(
                    Cr0Flags::EMULATE_COPROCESSOR | Cr0Flags::TASK_SWITCHED,
                );
                flags.insert(Cr0Flags::MONITOR_COPROCESSOR);
            });

            Cr4::update(|flags: &mut Cr4Flags| {
                flags.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE);
            });
        }
        log::debug!("SSE enabled");
    }
}

pub fn ticks() -> u64 { interrupts::get_ticks() }

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
