use alloc::vec::Vec;
use core::ptr::NonNull;

use acpi::{
    AcpiTables, PhysicalMapping,
    sdt::madt::{Madt, MadtEntry},
};
use spin::Once;
use x86_64::{PhysAddr, structures::paging::PageTableFlags};

use crate::{boot::limine::RSDP_REQUEST, system::mem::vmm};

#[derive(Clone, Debug)]
pub struct AcpiReader;

impl acpi::Handler for AcpiReader {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let virt_addr = vmm::phys_to_virt(PhysAddr::new(physical_address as u64));

        if !vmm::page_is_mapped(virt_addr)
            && let Err(e) = vmm::page_map(
                virt_addr,
                PhysAddr::new(physical_address as u64),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            )
        {
            panic!("failed to map physical region: {e}");
        }

        let virtual_start = NonNull::new(virt_addr.as_mut_ptr::<T>())
            .expect("acpi physical mapping translated to null virtual pointer");

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start,
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {
        // noop.
    }

    fn read_u8(&self, address: usize) -> u8 {
        todo!()
    }

    fn read_u16(&self, address: usize) -> u16 {
        todo!()
    }

    fn read_u32(&self, address: usize) -> u32 {
        todo!()
    }

    fn read_u64(&self, address: usize) -> u64 {
        todo!()
    }

    fn write_u8(&self, address: usize, value: u8) {
        todo!()
    }

    fn write_u16(&self, address: usize, value: u16) {
        todo!()
    }

    fn write_u32(&self, address: usize, value: u32) {
        todo!()
    }

    fn write_u64(&self, address: usize, value: u64) {
        todo!()
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        todo!()
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        todo!()
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        todo!()
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        todo!()
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        todo!()
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        todo!()
    }

    fn read_pci_u8(&self, address: acpi::PciAddress, offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, address: acpi::PciAddress, offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, address: acpi::PciAddress, offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(&self, address: acpi::PciAddress, offset: u16, value: u8) {
        todo!()
    }

    fn write_pci_u16(&self, address: acpi::PciAddress, offset: u16, value: u16) {
        todo!()
    }

    fn write_pci_u32(&self, address: acpi::PciAddress, offset: u16, value: u32) {
        todo!()
    }

    fn nanos_since_boot(&self) -> u64 {
        todo!()
    }

    fn stall(&self, microseconds: u64) {
        todo!()
    }

    fn sleep(&self, milliseconds: u64) {
        todo!()
    }

    fn create_mutex(&self) -> acpi::Handle {
        todo!()
    }

    fn acquire(&self, mutex: acpi::Handle, timeout: u16) -> Result<(), acpi::aml::AmlError> {
        todo!()
    }

    fn release(&self, mutex: acpi::Handle) {
        todo!()
    }
}

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

pub static ACPI_TABLES: Once<KernelAcpiTables> = Once::new();

pub fn install() {
    if ACPI_TABLES.get().is_some() {
        panic!("acpi tables already installed");
    }

    let mut tables = KernelAcpiTables::default();

    unsafe {
        let rsdp = RSDP_REQUEST
            .get_response()
            .expect("failed to get rsdp")
            .address();

        let acpi_tables =
            AcpiTables::from_rsdp(AcpiReader, rsdp).expect("failed to parse acpi tables");

        for madt in acpi_tables.find_tables::<Madt>() {
            for entry in madt.get().entries() {
                match entry {
                    MadtEntry::LocalApic(lapic) => {
                        tables.lapics.push(LapicInfo {
                            proc_id: lapic.processor_id,
                            apic_id: lapic.apic_id,
                            flags: lapic.flags,
                        });
                    },
                    MadtEntry::IoApic(ioapic) => tables.ioapics.push(IoApicInfo {
                        id: ioapic.io_apic_id,
                        address: ioapic.io_apic_address,
                    }),
                    _ => {},
                }
            }
        }
    }

    ACPI_TABLES.call_once(|| tables);
}

pub fn get() -> &'static KernelAcpiTables {
    ACPI_TABLES.get().expect("acpi tables not installed")
}
