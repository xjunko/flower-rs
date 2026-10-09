use acpi::AcpiTables;
use spin::once::Once;
use x86_64::VirtAddr;

use crate::arch::acpi::parser::AcpiReader;
use crate::arch::acpi::tables::KernelAcpiTables;
use crate::boot::limine::RSDP_REQUEST;
use crate::system::mem;

mod parser;
mod tables;

pub static ACPI_TABLES: Once<KernelAcpiTables> = Once::new();

pub fn install() {
    let mut tables = KernelAcpiTables::default();

    log::debug!("ACPI: searching for RSDP...");
    if let Some(rsdp) = RSDP_REQUEST.response() {
        log::debug!("ACPI: RSDP found at {:#x}", rsdp.address as usize);

        let virt_addr = VirtAddr::new(rsdp.address as u64);
        let phys_addr = mem::vmm::virt_to_phys(virt_addr)
            .expect("failed to get phys addr of rsdp");

        unsafe {
            if let Ok(acpi) =
                AcpiTables::from_rsdp(AcpiReader, phys_addr.as_u64() as usize)
            {
                tables.parse_madt(&acpi);
            } else {
                panic!("failed to parse acpi tables");
            }
        }
    } else {
        panic!("ACPI: RSDP not found");
    }

    ACPI_TABLES.call_once(|| tables);
}

pub fn get() -> &'static KernelAcpiTables {
    ACPI_TABLES.get().expect("acpi tables not initialized")
}
