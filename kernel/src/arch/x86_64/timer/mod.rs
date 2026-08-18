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

use core::arch::x86_64::__cpuid;

use spin::Once;

pub mod acpi_pmt;
pub mod tsc;

#[derive(Debug, Clone, Copy)]
pub enum TimerType {
    Tsc,
    AcpiPmt,
    Undefined,
}

static TIMER_TYPE: Once<TimerType> = Once::new();
static TIMER_FREQ: Once<u64> = Once::new();
static TIMER_TICKS_INIT: Once<u64> = Once::new();

fn __has_invariant_tsc() -> bool {
    const INVARIANT_TSC: u32 = 1 << 8;

    let max_ext = __cpuid(0x8000_0000).eax;
    if max_ext < 0x8000_0007 {
        return false;
    }
    (__cpuid(0x8000_0007).edx & INVARIANT_TSC) != 0
}

fn __shutup_clippy() -> bool { false }
pub fn install() {
    let mut freq = 0u64;
    let mut typ = TimerType::Undefined;

    if __has_invariant_tsc() {
        freq = tsc::measure_tsc_freq();
        typ = TimerType::Tsc;
    } else if __shutup_clippy() {
    } else {
        freq = acpi_pmt::ACPI_TIMER_FREQUENCY;
        typ = TimerType::AcpiPmt;
    }

    TIMER_TYPE.call_once(|| typ);
    TIMER_FREQ.call_once(|| freq);
    TIMER_TICKS_INIT.call_once(self::get_ticks);

    log::debug!("timer type: {:?}, frequency: {}hz", typ, freq);
}

pub fn get_ticks() -> u64 {
    match TIMER_TYPE.get() {
        Some(TimerType::Tsc) => unsafe { core::arch::x86_64::_rdtsc() },
        Some(TimerType::AcpiPmt) => acpi_pmt::get_ticks(),
        _ => unreachable!(),
    }
}

pub fn get_ns() -> u64 {
    let ticks = get_ticks();
    let ticks_started_at =
        *TIMER_TICKS_INIT.get().expect("timer start time not initialized");
    let freq = *TIMER_FREQ.get().expect("timer frequency not initialized");
    let elapsed = ticks.wrapping_sub(ticks_started_at);
    ((elapsed as u128 * 1_000_000_000) / freq as u128) as u64
}
