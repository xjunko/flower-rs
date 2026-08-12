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

use alloc::vec::Vec;

use acpi::AcpiTables;
use acpi::sdt::madt::{Madt, MadtEntry};
use x86_64::VirtAddr;

use crate::acpi::parser::KernelAcpiReader;

#[derive(Debug)]
pub struct LapicInfo {
    pub proc_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

#[derive(Debug)]
pub struct IoApicInfo {
    pub id: u8,
    pub address: u32,
}

#[derive(Debug)]
pub struct KernelAcpiTables {
    pub lapic_base: VirtAddr,
    pub lapics: Vec<LapicInfo>,
    pub ioapics: Vec<IoApicInfo>,
}

impl Default for KernelAcpiTables {
    fn default() -> Self {
        Self {
            lapic_base: VirtAddr::new(0),
            lapics: Vec::new(),
            ioapics: Vec::new(),
        }
    }
}

impl KernelAcpiTables {
    pub fn parse_madt(&mut self, acpi: &AcpiTables<KernelAcpiReader>) {
        for madt in acpi.find_tables::<Madt>() {
            self.lapic_base =
                VirtAddr::new(madt.get().local_apic_address as u64);

            for entry in madt.get().entries() {
                match entry {
                    MadtEntry::LocalApic(lapic) => {
                        self.lapics.push(LapicInfo {
                            proc_id: lapic.processor_id,
                            apic_id: lapic.apic_id,
                            flags: lapic.flags,
                        });
                    },
                    MadtEntry::IoApic(ioapic) => {
                        self.ioapics.push(IoApicInfo {
                            id: ioapic.io_apic_id,
                            address: ioapic.io_apic_address,
                        })
                    },
                    _ => {},
                }
            }
        }
    }
}
