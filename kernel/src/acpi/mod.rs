use acpi::AcpiTables;
use spin::once::Once;
use x86_64::VirtAddr;

use crate::acpi::parser::KernelAcpiReader;
use crate::acpi::tables::KernelAcpiTables;
use crate::boot::limine::RSDP_REQUEST;
use crate::system::mem::vmm::AddressSpace;

mod parser;
mod tables;

pub static ACPI_TABLES: Once<KernelAcpiTables> = Once::new();

pub fn install() {
    let mut tables = KernelAcpiTables::default();

    log::debug!("acpi: searching for rsdp...");
    if let Some(rsdp) = RSDP_REQUEST.response() {
        let virt_addr = VirtAddr::new(rsdp.address as u64);
        let phys_addr = AddressSpace::virt_to_phys(virt_addr.as_u64())
            .expect("failed to convert rsdp to phys");

        log::debug!("acpi: rsdp found at {:#x}", virt_addr);

        unsafe {
            if let Ok(acpi) =
                AcpiTables::from_rsdp(KernelAcpiReader, phys_addr as usize)
            {
                tables.parse_madt(&acpi);
                tables.parse_fadt(&acpi);
            } else {
                panic!("failed to parse acpi tables");
            }
        }
    } else {
        panic!("acpi: rsdp not found");
    }

    ACPI_TABLES.call_once(|| tables);
}

pub fn get() -> &'static KernelAcpiTables {
    ACPI_TABLES.get().expect("acpi tables not initialized")
}
