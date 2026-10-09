use core::ptr::NonNull;

use acpi::PhysicalMapping;
use x86_64::PhysAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::system;

#[derive(Clone, Debug)]
pub struct AcpiReader;

impl acpi::Handler for AcpiReader {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let virt_addr = system::mem::vmm::phys_to_virt(PhysAddr::new(
            physical_address as u64,
        ));

        if !system::mem::vmm::page_is_mapped(virt_addr)
            && let Err(e) = system::mem::vmm::page_map(
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
