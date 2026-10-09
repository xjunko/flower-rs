use x86_64::VirtAddr;

use crate::arch::MapFlags;
use crate::system::mem::vmm::AddressSpace;
use crate::{acpi, arch};

const IOAPIC_REDIR_TABLE: u32 = 0x10;
const IOAPIC_SIZE: usize = arch::layout::PAGE_SIZE;

pub struct IoApic {
    virt_base: VirtAddr,
    gsi_base: u32,
}

impl IoApic {
    const REG_SEL: u64 = 0x00;
    const REG_WIN: u64 = 0x10;

    pub fn install(address_space: &AddressSpace) -> Self {
        let virt = AddressSpace::reserve_virt(IOAPIC_SIZE)
            .expect("failed to reserve virt for ioapic");

        let acpi_tables = acpi::get();
        if acpi_tables.ioapics.is_empty() {
            panic!("no ioapic found in acpi tables");
        }

        let ioapic_addr = acpi_tables.ioapics[0].address;
        let ioapic_gsi_base = acpi_tables.ioapics[0].gsi_base;
        log::debug!(
            "ioapic addr: {:#x}, gsi_base: {}",
            ioapic_addr,
            ioapic_gsi_base
        );

        let flags = MapFlags::WRITE | MapFlags::NO_CACHE;

        address_space
            .map_page(virt, ioapic_addr as u64, flags)
            .expect("failed to map ioapic");

        Self { virt_base: VirtAddr::new(virt), gsi_base: ioapic_gsi_base }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        let base = self.virt_base.as_u64();

        unsafe {
            core::ptr::write_volatile((base + Self::REG_SEL) as *mut u32, reg);
            core::ptr::read((base + Self::REG_WIN) as *const u32)
        }
    }

    unsafe fn write(&self, reg: u32, value: u32) {
        let base = self.virt_base.as_u64();

        unsafe {
            core::ptr::write_volatile((base + Self::REG_SEL) as *mut u32, reg);
            core::ptr::write_volatile(
                (base + Self::REG_WIN) as *mut u32,
                value,
            );
        }
    }

    pub fn set_redirection(&self, gsi: u32, vector: u8, dest_apic_id: u8) {
        assert!(gsi >= self.gsi_base);

        let idx = gsi - self.gsi_base;
        let redir = IOAPIC_REDIR_TABLE + idx * 2;

        unsafe {
            self.write(redir + 1, u32::from(dest_apic_id) << 24);
            self.write(redir, u32::from(vector));
        }
    }
}
