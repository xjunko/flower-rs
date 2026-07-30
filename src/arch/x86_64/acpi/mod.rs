use acpi::AcpiTables;
use spin::once::Once;

use crate::arch::acpi::parser::AcpiReader;
use crate::arch::acpi::tables::KernelAcpiTables;
use crate::boot::limine::RSDP_REQUEST;

mod parser;
mod tables;

pub static ACPI_TABLES: Once<KernelAcpiTables> = Once::new();

pub fn install() {
    let mut tables = KernelAcpiTables::default();

    log::debug!("ACPI: searching for RSDP...");
    if let Some(rsdp) = RSDP_REQUEST.get_response() {
        log::debug!("ACPI: RSDP found at {:#x}", rsdp.address());

        unsafe {
            if let Ok(acpi) = AcpiTables::from_rsdp(AcpiReader, rsdp.address())
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
