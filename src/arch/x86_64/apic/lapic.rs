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

use core::sync::atomic::{AtomicU32, Ordering};

use x86_64::VirtAddr;
use x86_64::registers::model_specific::{ApicBase, ApicBaseFlags};
use x86_64::structures::paging::PageTableFlags;

use crate::arch::x86_64::timer::acpi_pmt;
use crate::memory::vmm::AddressSpace;

const LAPIC_EOI: u64 = 0x0B0;
const LAPIC_SPURIOUS: u64 = 0x0F0;
const LAPIC_TIMER_LVT: u64 = 0x320;
const LAPIC_TIMER_INIT: u64 = 0x380;
const LAPIC_TIMER_CURRENT: u64 = 0x390;
const LAPIC_TIMER_DIV: u64 = 0x3E0;
const LAPIC_ID: u64 = 0x020;

const CALIBRATE_MS: u32 = 10;

pub struct LocalApic {
    virt_base: VirtAddr,
    ticks_per_ms: AtomicU32,
}

impl LocalApic {
    pub fn init(address_space: &AddressSpace, virt: VirtAddr) -> Self {
        let (apic_base, apic_flags) = ApicBase::read();
        log::debug!("apic addr: {:#x}", apic_base.start_address().as_u64());

        if !apic_flags.contains(ApicBaseFlags::LAPIC_ENABLE) {
            log::debug!("lapic not enabled, enabling");
            unsafe {
                ApicBase::write(
                    apic_base,
                    apic_flags | ApicBaseFlags::LAPIC_ENABLE,
                );
            }
        } else {
            log::debug!("lapic already enabled");
        }

        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE;
        address_space
            .map_page(virt, apic_base.start_address(), flags)
            .expect("failed to map lapic");

        Self { virt_base: virt, ticks_per_ms: AtomicU32::new(0) }
    }

    unsafe fn read(&self, offset: u64) -> u32 {
        unsafe {
            core::ptr::read_volatile(
                (self.virt_base.as_u64() + offset) as *const u32,
            )
        }
    }

    unsafe fn write(&self, offset: u64, value: u32) {
        unsafe {
            core::ptr::write_volatile(
                (self.virt_base.as_u64() + offset) as *mut u32,
                value,
            )
        }
    }

    pub fn id(&self) -> u8 { (unsafe { self.read(LAPIC_ID) } >> 24) as u8 }

    pub fn calibrate(&self) {
        unsafe {
            self.write(LAPIC_TIMER_DIV, 0x3);
            self.write(LAPIC_TIMER_INIT, 0xFFFFFFFF);

            acpi_pmt::wait_ms(CALIBRATE_MS);

            let elapsed = 0xFFFFFFFF - self.read(LAPIC_TIMER_CURRENT);
            self.write(LAPIC_TIMER_INIT, 0);

            let ticks_ms = elapsed / CALIBRATE_MS;
            self.ticks_per_ms.store(ticks_ms, Ordering::Relaxed);
            log::debug!("calibrated lapic timer: {} ticks/ms", ticks_ms);
        }
    }
}

impl LocalApic {
    pub fn enable_spurious(&self, vector: u8) {
        unsafe { self.write(LAPIC_SPURIOUS, 0x100 | vector as u32) };
    }

    pub fn start_periodic_timer(&self, vector: u8) {
        let ticks_ms = self.ticks_per_ms.load(Ordering::Relaxed);
        unsafe {
            self.write(LAPIC_TIMER_DIV, 0x3);
            self.write(LAPIC_TIMER_LVT, (1 << 17) | vector as u32);
            self.write(LAPIC_TIMER_INIT, ticks_ms);
        }
    }

    pub fn eoi(&self) { unsafe { self.write(LAPIC_EOI, 0) }; }
}
