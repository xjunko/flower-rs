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

mod apic;
pub mod gdt;
mod idt;
mod interrupts;
pub mod layout;
mod timer;

use core::arch::asm;

use raw_cpuid::CpuId;
use x86_64::VirtAddr;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

use crate::arch::{Arch, Paging, Processor};

fn install_cpu_features() {
    let cpuid = CpuId::new();
    if let Some(finfo) = cpuid.get_feature_info() {
        assert!(finfo.has_fxsave_fxstor(), "fxsave/fxstor not supported");
        assert!(finfo.has_mmx(), "mmx not supported");
        assert!(finfo.has_sse(), "sse not supported");
        assert!(finfo.has_fpu(), "fpu not supported");

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
        log::debug!("sse enabled");
    }
}

impl Arch for Processor {
    fn bsp_install() {
        self::install_cpu_features();
        self::gdt::install();
        self::idt::install();
    }

    fn ap_install() { todo!() }

    fn paging_install() { todo!() }

    fn timer_install() {
        apic::install();
        timer::install();
    }

    fn timer_get_ns() -> u64 { timer::get_ns() }

    fn halt() -> ! {
        loop {
            unsafe { asm!("hlt") }
        }
    }

    fn write<T>(_p: u32, _d: T) { todo!() }

    fn read<T>(_p: u32) -> T { todo!() }

    fn set_kernel_stack(v: u64) { gdt::set_kernel_stack(VirtAddr::new(v)) }

    fn interrupts_enable() { x86_64::instructions::interrupts::enable() }

    fn interrupts_disable() { x86_64::instructions::interrupts::disable() }

    fn interrupts_ack() { apic::eoi() }
}

impl Paging for Processor {
    type Root = u64;

    const KERNEL_HALF_START: u64 = 0xffff_8000_0000_0000;
    const PAGE_SIZE: usize = 0x1000;

    fn hhdm_offset() -> u64 { todo!() }

    fn active_root() -> Self::Root { todo!() }

    fn set_active_root(root: Self::Root) { todo!() }

    fn root_new() -> Result<Self::Root, &'static str> { todo!() }

    fn root_free(root: Self::Root) { todo!() }

    fn root_phys(root: Self::Root) -> u64 { todo!() }

    fn map(
        root: Self::Root,
        virt: u64,
        phys: u64,
        flags: super::MapFlags,
    ) -> Result<(), &'static str> {
        todo!()
    }

    fn unmap(root: Self::Root, virt: u64) -> Result<u64, &'static str> {
        todo!()
    }

    fn protect(
        root: Self::Root,
        virt: u64,
        flags: super::MapFlags,
    ) -> Result<(), &'static str> {
        todo!()
    }

    fn translate(
        root: Self::Root,
        virt: u64,
    ) -> Option<(u64, super::MapFlags)> {
        todo!()
    }

    fn tlb_flush(virt: u64) { todo!() }

    fn tlb_flush_all() { todo!() }

    fn clone_user(root: Self::Root) -> Result<Self::Root, &'static str> {
        todo!()
    }
}
