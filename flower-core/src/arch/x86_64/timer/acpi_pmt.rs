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

use x86_64::instructions::port::Port;

use crate::acpi;

pub const ACPI_TIMER_FREQUENCY: u64 = 3579545;

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

static LAST_RAW: AtomicU32 = AtomicU32::new(0);
static ACCUM: AtomicU64 = AtomicU64::new(0);

fn raw_get_ticks() -> u32 {
    let table = acpi::get();
    if let Some(pm_timer) = table.pm_timer_block_addr {
        let mut port = Port::<u32>::new(pm_timer.as_u64() as u16);
        return unsafe { port.read() & 0xFFFFFF };
    }

    unreachable!()
}

pub fn get_ticks() -> u64 {
    let raw = raw_get_ticks();
    let last = LAST_RAW.swap(raw, Ordering::AcqRel);
    let delta = raw.wrapping_sub(last) & 0xFFFFFF;
    ACCUM.fetch_add(delta as u64, Ordering::AcqRel) + delta as u64
}

pub fn wait_ms(ms: u32) {
    let ticks_needed: u64 = (ACPI_TIMER_FREQUENCY * ms as u64 + 999) / 1000;
    let start: u64 = self::get_ticks();
    while (self::get_ticks() - start) < ticks_needed {
        core::hint::spin_loop();
    }
}
