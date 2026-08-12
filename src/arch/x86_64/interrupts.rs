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

use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::instructions::interrupts;
use x86_64::structures::idt::InterruptStackFrame;

use crate::arch::x86_64::apic;
use crate::system::proc;

static TICKS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Keyboard = 33,
    Spurious = 255,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 { self as u8 }

    pub fn as_usize(self) -> usize { usize::from(self.as_u8()) }
}

pub fn enable() { interrupts::enable(); }

pub fn disable() { interrupts::disable(); }

pub fn get_ticks() -> u64 { TICKS.load(Ordering::Relaxed) }

pub extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    apic::eoi();
    proc::schedule();
}

pub extern "x86-interrupt" fn spurious_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
}
