use alloc::vec::Vec;

use acpi::AcpiTables;
use acpi::sdt::madt::{Madt, MadtEntry};

use crate::arch::acpi::parser::AcpiReader;

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

#[derive(Debug, Default)]
pub struct KernelAcpiTables {
    pub lapics: Vec<LapicInfo>,
    pub ioapics: Vec<IoApicInfo>,
}

impl KernelAcpiTables {
    pub fn parse_madt(&mut self, acpi: &AcpiTables<AcpiReader>) {
        for madt in acpi.find_tables::<Madt>() {
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
