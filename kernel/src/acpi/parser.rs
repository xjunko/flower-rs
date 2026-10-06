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

use core::ptr::NonNull;

use acpi::PhysicalMapping;
use x86_64::PhysAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::memory::vmm::AddressSpace;

#[derive(Clone, Debug)]
pub struct KernelAcpiReader;

impl acpi::Handler for KernelAcpiReader {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let address_space = AddressSpace::kernel();
        let phys_start = PhysAddr::new(physical_address as u64);
        let virt_start = AddressSpace::phys_to_virt(phys_start);

        // we're gonna map the whole region
        let page_offset = (physical_address as u64 & 0xFFF) as usize;
        let aligned_phys = physical_address as u64 - page_offset as u64;
        let total_len = size + page_offset;
        let page_count = total_len.div_ceil(4096);

        for i in 0..page_count {
            let page_phys = PhysAddr::new(aligned_phys + (i as u64 * 4096));
            let page_virt = AddressSpace::phys_to_virt(page_phys);

            if !address_space.is_mapped(page_virt)
                && let Err(e) = address_space.map_page(
                    page_virt,
                    page_phys,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                )
            {
                panic!("failed to map physical page: {e}")
            }
        }

        let virtual_start = NonNull::new(virt_start.as_mut_ptr::<T>())
            .expect("acpi physical mapping translated to null virtual pointer");

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start,
            region_length: size,
            mapped_length: size,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
        // noop.
    }

    fn read_u8(&self, _address: usize) -> u8 { todo!() }

    fn read_u16(&self, _address: usize) -> u16 { todo!() }

    fn read_u32(&self, _address: usize) -> u32 { todo!() }

    fn read_u64(&self, _address: usize) -> u64 { todo!() }

    fn write_u8(&self, _address: usize, _value: u8) { todo!() }

    fn write_u16(&self, _address: usize, _value: u16) { todo!() }

    fn write_u32(&self, _address: usize, _value: u32) { todo!() }

    fn write_u64(&self, _address: usize, _svalue: u64) { todo!() }

    fn read_io_u8(&self, _port: u16) -> u8 { todo!() }

    fn read_io_u16(&self, _port: u16) -> u16 { todo!() }

    fn read_io_u32(&self, _port: u16) -> u32 { todo!() }

    fn write_io_u8(&self, _port: u16, _value: u8) { todo!() }

    fn write_io_u16(&self, _port: u16, _value: u16) { todo!() }

    fn write_io_u32(&self, _port: u16, _value: u32) { todo!() }

    fn read_pci_u8(&self, _address: acpi::PciAddress, _offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, _address: acpi::PciAddress, _offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, _address: acpi::PciAddress, _offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(
        &self,
        _address: acpi::PciAddress,
        _offset: u16,
        _value: u8,
    ) {
        todo!()
    }

    fn write_pci_u16(
        &self,
        _address: acpi::PciAddress,
        _offset: u16,
        _value: u16,
    ) {
        todo!()
    }

    fn write_pci_u32(
        &self,
        _address: acpi::PciAddress,
        _offset: u16,
        _value: u32,
    ) {
        todo!()
    }

    fn nanos_since_boot(&self) -> u64 { todo!() }

    fn stall(&self, _microseconds: u64) { todo!() }

    fn sleep(&self, _milliseconds: u64) { todo!() }

    fn create_mutex(&self) -> acpi::Handle { todo!() }

    fn acquire(
        &self,
        _mutex: acpi::Handle,
        _timeout: u16,
    ) -> Result<(), acpi::aml::AmlError> {
        todo!()
    }

    fn release(&self, _mutex: acpi::Handle) { todo!() }
}
