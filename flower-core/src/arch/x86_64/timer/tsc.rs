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

use raw_cpuid::CpuId;

use crate::arch::x86_64::timer::acpi_pmt;

fn __measure_tsc_freq() -> u64 {
    let mut total: u64 = 0;
    for _ in 0..3 {
        let start = unsafe { core::arch::x86_64::_rdtsc() };
        acpi_pmt::wait_ms(10);
        let end = unsafe { core::arch::x86_64::_rdtsc() };
        total += (end - start) * 100;
    }

    total / 3
}

pub fn measure_tsc_freq() -> u64 {
    let cpuid = CpuId::new();

    // intel only
    if let Some(vendor) = cpuid.get_vendor_info() {
        match vendor.as_str() {
            "GenuineIntel" | "GenuineIotel" => {
                if let Some(tsc_info) = cpuid.get_tsc_info()
                    && let Some(tsc_freq) = tsc_info.tsc_frequency()
                {
                    return tsc_freq;
                }
            },
            _ => {},
        }
    }

    let freq = __measure_tsc_freq();
    log::info!("tsc frequency: {}hz", freq);
    freq
}
