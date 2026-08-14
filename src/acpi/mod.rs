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

use acpi::AcpiTables;
use spin::once::Once;

use crate::acpi::parser::KernelAcpiReader;
use crate::acpi::tables::KernelAcpiTables;
use crate::boot::limine::RSDP_REQUEST;

mod parser;
mod tables;

pub static ACPI_TABLES: Once<KernelAcpiTables> = Once::new();

pub fn install() {
    let mut tables = KernelAcpiTables::default();

    log::debug!("acpi: searching for rsdp...");
    if let Some(rsdp) = RSDP_REQUEST.get_response() {
        log::debug!("acpi: rsdp found at {:#x}", rsdp.address());

        unsafe {
            if let Ok(acpi) =
                AcpiTables::from_rsdp(KernelAcpiReader, rsdp.address())
            {
                tables.parse_madt(&acpi);
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
