use alloc::vec::Vec;

use acpi::AcpiTables;
use acpi::sdt::fadt::Fadt;
use acpi::sdt::madt::{Madt, MadtEntry};
use x86_64::{PhysAddr, VirtAddr};

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
    pub gsi_base: u32,
}

#[derive(Debug)]
pub struct InterruptSourceOverrideInfo {
    pub bus: u8,
    pub irq: u8,
    pub gsi: u32,
    pub flags: u16,
}

#[derive(Debug)]
pub struct KernelAcpiTables {
    pub lapic_base: VirtAddr,
    pub lapics: Vec<LapicInfo>,
    pub ioapics: Vec<IoApicInfo>,
    pub interrupt_overrides: Vec<InterruptSourceOverrideInfo>,

    pub pm_timer_block_addr: Option<PhysAddr>,
}

impl Default for KernelAcpiTables {
    fn default() -> Self {
        Self {
            lapic_base: VirtAddr::new(0),
            lapics: Vec::new(),
            ioapics: Vec::new(),
            interrupt_overrides: Vec::new(),
            pm_timer_block_addr: None,
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
                            gsi_base: ioapic.global_system_interrupt_base,
                        })
                    },
                    MadtEntry::InterruptSourceOverride(iso) => {
                        self.interrupt_overrides.push(
                            InterruptSourceOverrideInfo {
                                bus: iso.bus,
                                irq: iso.irq,
                                gsi: iso.global_system_interrupt,
                                flags: iso.flags,
                            },
                        );
                    },
                    _ => {},
                }
            }
        }
    }

    pub fn parse_fadt(&mut self, acpi: &AcpiTables<KernelAcpiReader>) {
        for fadt in acpi.find_tables::<Fadt>() {
            if let Some(pm_timer) = fadt.pm_timer_block().unwrap() {
                self.pm_timer_block_addr =
                    Some(PhysAddr::new(pm_timer.address));

                log::debug!("acpi: pm timer found at: {:#x}", pm_timer.address);
            }
        }

        if self.pm_timer_block_addr.is_none() {
            panic!("acpi: fadt does not contain pm timer block address");
        }
    }
}

impl KernelAcpiTables {
    pub fn irq_to_gsi(&self, irq: u8) -> u32 {
        if let Some(iso) =
            self.interrupt_overrides.iter().find(|iso| iso.irq == irq)
        {
            return iso.gsi;
        }

        self.ioapics
            .iter()
            .find(|ioapic| {
                let end = ioapic.gsi_base + 24;
                u32::from(irq) >= ioapic.gsi_base && u32::from(irq) < end
            })
            .map(|ioapic| ioapic.gsi_base + u32::from(irq))
            .expect("no ioapic for irq")
    }
}
