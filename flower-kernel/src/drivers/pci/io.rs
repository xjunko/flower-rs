use pci_types::ConfigRegionAccess;
use x86_64::instructions::port::Port;

const CMD_PORT: u16 = 0xCF8;
const DATA_PORT: u16 = 0xCFC;

pub struct PciIO;

impl ConfigRegionAccess for PciIO {
    unsafe fn read(&self, address: acpi::PciAddress, offset: u16) -> u32 {
        let addr: u32 = (1 << 31)
            | ((address.bus() as u32) << 16)
            | ((address.device() as u32) << 11)
            | ((address.function() as u32) << 8)
            | ((offset as u32) & 0xFC);

        let mut cmd = Port::<u32>::new(CMD_PORT);
        let mut data = Port::<u32>::new(DATA_PORT);

        unsafe {
            cmd.write(addr);
            data.read()
        }
    }

    unsafe fn write(&self, address: acpi::PciAddress, offset: u16, value: u32) {
        let addr: u32 = (1 << 31)
            | ((address.bus() as u32) << 16)
            | ((address.device() as u32) << 11)
            | ((address.function() as u32) << 8)
            | ((offset as u32) & 0xFC);

        let mut cmd = Port::<u32>::new(CMD_PORT);
        let mut data = Port::<u32>::new(DATA_PORT);

        unsafe {
            cmd.write(addr);
            data.write(value)
        }
    }
}
