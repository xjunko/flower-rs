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
mod features;
pub mod gdt;
mod idt;
mod interrupts;
pub mod layout;
mod multitasking;
mod paging;
pub mod port;
mod timer;

use core::arch::asm;

use crate::arch::{Arch, Port, Processor};

static TIMER_INSTALLED: spin::Once<()> = spin::Once::new();

impl Arch for Processor {
    fn bsp_install() {
        self::features::install();
        self::gdt::install();
        self::idt::install();
    }

    fn ap_install() { todo!() }

    fn paging_install() { self::paging::install() }

    fn timer_install() {
        apic::install();
        timer::install();
        TIMER_INSTALLED.call_once(|| {});
    }

    fn timer_get_ns() -> u64 {
        if TIMER_INSTALLED.is_completed() { timer::get_ns() } else { 0 }
    }

    fn halt() -> ! {
        loop {
            unsafe { asm!("hlt") }
        }
    }

    fn write<T: Port>(p: u32, d: T) { self::port::write::<T>(p as u16, d) }

    fn read<T: Port>(p: u32) -> T { self::port::read::<T>(p as u16) }

    fn set_kernel_stack(v: u64) { gdt::set_kernel_stack(v) }

    fn interrupts_enable() { x86_64::instructions::interrupts::enable() }

    fn interrupts_disable() { x86_64::instructions::interrupts::disable() }

    fn interrupts_ack() { apic::eoi() }

    fn interrupts_enabled() -> bool {
        x86_64::instructions::interrupts::are_enabled()
    }
}
