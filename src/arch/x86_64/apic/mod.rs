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

pub mod ioapic;
pub mod lapic;

use raw_cpuid::CpuId;
use spin::Once;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

use crate::arch::x86_64::apic::ioapic::IoApic;
use crate::arch::x86_64::apic::lapic::LocalApic;
use crate::arch::x86_64::interrupts::InterruptIndex;
use crate::arch::x86_64::layout::{IOAPIC_VIRT, LAPIC_VIRT};
use crate::memory::vmm::AddressSpace;

// legacy pic
fn pic_disable() {
    const PIC1: u16 = 0x20;
    const PIC1_DATA: u16 = PIC1 + 1;

    const PIC2: u16 = 0xA0;
    const PIC2_DATA: u16 = PIC2 + 1;

    interrupts::without_interrupts(|| {
        let mut p1_data: Port<u8> = Port::new(PIC1_DATA);
        let mut p2_data: Port<u8> = Port::new(PIC2_DATA);

        unsafe {
            p1_data.write(0xFF);
            p2_data.write(0xFF);
        }
    })
}

pub struct Apic {
    pub lapic: LocalApic,
    pub ioapic: IoApic,
}

static APIC: Once<Apic> = Once::new();

pub fn install() {
    pic_disable();

    let cpuid = CpuId::new();
    let address_space = AddressSpace::current();

    let finfo =
        cpuid.get_feature_info().expect("cpuid feature info unavailable");

    if finfo.has_x2apic() {
        log::warn!("x2apic supported but unused, falling back to xapic");
    }
    if !finfo.has_apic() {
        panic!("cpu does not support apic");
    }

    let lapic = LocalApic::init(&address_space, VirtAddr::new(LAPIC_VIRT));
    let ioapic = IoApic::init(&address_space, VirtAddr::new(IOAPIC_VIRT));

    lapic.enable_spurious(InterruptIndex::Spurious as u8);
    lapic.calibrate();
    lapic.start_periodic_timer(InterruptIndex::Timer as u8);

    ioapic.set_redirection(1, InterruptIndex::Keyboard as u8, lapic.id());

    APIC.call_once(|| Apic { lapic, ioapic });
}

pub fn eoi() { APIC.get().expect("apic not installed").lapic.eoi(); }
