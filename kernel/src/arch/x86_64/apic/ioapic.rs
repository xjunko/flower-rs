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

use x86_64::structures::paging::PageTableFlags;
use x86_64::{PhysAddr, VirtAddr};

use crate::acpi;
use crate::memory::vmm::AddressSpace;

const IOAPIC_REG_DATA: u64 = 0x10;
const IOAPIC_REDIR_TABLE: u32 = 0x10;

pub struct IoApic {
    virt_base: VirtAddr,
}

impl IoApic {
    pub fn init(address_space: &AddressSpace, virt: VirtAddr) -> Self {
        let acpi_tables = acpi::get();
        if acpi_tables.ioapics.is_empty() {
            panic!("no ioapic found in acpi tables");
        }
        let ioapic_addr = acpi_tables.ioapics[0].address;
        log::debug!("ioapic addr: {:#x}", ioapic_addr);

        let flags = PageTableFlags::PRESENT
            | PageTableFlags::WRITABLE
            | PageTableFlags::NO_CACHE;
        address_space
            .map_page(virt, PhysAddr::new(ioapic_addr as u64), flags)
            .expect("failed to map ioapic");

        Self { virt_base: virt }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        unsafe {
            core::ptr::write_volatile(self.virt_base.as_u64() as *mut u32, reg);
            core::ptr::read_volatile(
                (self.virt_base.as_u64() + IOAPIC_REG_DATA) as *const u32,
            )
        }
    }

    unsafe fn write(&self, reg: u32, value: u32) {
        unsafe {
            core::ptr::write_volatile(self.virt_base.as_u64() as *mut u32, reg);
            core::ptr::write_volatile(
                (self.virt_base.as_u64() + IOAPIC_REG_DATA) as *mut u32,
                value,
            );
        }
    }

    pub fn set_redirection(&self, irq: u8, vector: u8, dest_apic_id: u8) {
        let redir = IOAPIC_REDIR_TABLE + (u32::from(irq) * 2);
        unsafe {
            self.write(redir + 1, u32::from(dest_apic_id) << 24);
            self.write(redir, u32::from(vector));
        }
    }
}
