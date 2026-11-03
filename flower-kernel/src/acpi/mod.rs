use acpi::AcpiTables;
use spin::Once;

use crate::acpi::parser::AcpiReader;
use crate::acpi::tables::KernelAcpiReader;
use crate::boot::limine::RSDP_REQUEST;

mod parser;
mod tables;

pub static ACPI_TABLES: Once<KernelAcpiReader> = Once::new();

pub fn install() {
    let mut tables = KernelAcpiReader::default();

    log::debug!("ACPI: searching for RSDP...");
    if let Some(rsdp) = RSDP_REQUEST.get_response() {
        log::debug!("ACPI: RSDP found at {:#x}", rsdp.address());

        unsafe {
            if let Ok(acpi) = AcpiTables::from_rsdp(AcpiReader, rsdp.address())
            {
                tables.parse_madt(&acpi);
                tables.parse_fadt(&acpi);
            } else {
                panic!("failed to parse acpi tables");
            }
        }
    } else {
        panic!("ACPI: RSDP not found");
    }

    ACPI_TABLES.call_once(|| tables);
}

pub fn get() -> &'static KernelAcpiReader {
    ACPI_TABLES.get().expect("acpi tables not initialized")
}
