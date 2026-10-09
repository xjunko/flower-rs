use alloc::vec::Vec;

use acpi::PciAddress;
use pci_types::{Bar, EndpointHeader, HeaderType, PciHeader};

use crate::drivers::pci::io::PciIO;

#[derive(Debug, Clone)]
pub struct PciDevice {
    pub addr: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision: u8,
    pub base_class: u8,
    pub sub_class: u8,
    pub interface: u8,
    pub bars: [Option<Bar>; 6],
}

pub struct PciBus {
    pub devices: Vec<PciDevice>,
}

impl PciBus {
    pub fn new() -> Self { Self { devices: Vec::new() } }

    pub fn parse(&mut self) {
        let pci = PciIO;
        for bus in 0..=255 {
            for device in 0..32 {
                let addr0 = PciAddress::new(0, bus, device, 0);
                let header0 = PciHeader::new(addr0);
                let (device_id, _) = header0.id(&pci);

                if device_id == 0xFFFF {
                    continue;
                }

                let functions =
                    if header0.has_multiple_functions(&pci) { 8 } else { 1 };

                for function in 0..functions {
                    let addr = PciAddress::new(0, bus, device, function);
                    let header = PciHeader::new(addr);
                    let (vendor_id, device_id) = header.id(&pci);

                    if vendor_id == 0xFFFF {
                        continue;
                    }

                    let (revision, base_class, sub_class, interface) =
                        header.revision_and_class(&pci);

                    let mut bars: [Option<Bar>; 6] = Default::default();

                    let header_type = header.header_type(&pci);
                    match header_type {
                        HeaderType::Endpoint => {
                            let endp =
                                EndpointHeader::from_header(header, &pci)
                                    .unwrap();
                            for (i, item) in bars.iter_mut().enumerate() {
                                *item = endp.bar(i as u8, &pci);
                            }
                        },

                        _ => {
                            unimplemented!(
                                "unsupported header: {:?}",
                                header_type
                            )
                        },
                    }

                    self.devices.push(PciDevice {
                        addr,
                        vendor_id,
                        device_id,
                        revision,
                        base_class,
                        sub_class,
                        interface,
                        bars,
                    });
                }
            }
        }

        log::info!("PCI found {} devices.", self.devices.len());
    }

    /// finds the first device matching the given class and subclass. returns None if no such device exists.
    pub fn find_by_class(&self, class: u8, subclass: u8) -> Option<&PciDevice> {
        self.devices
            .iter()
            .find(|dev| dev.base_class == class && dev.sub_class == subclass)
    }

    /// finds the first device matching the given vendor and device id. returns None if no such device exists.
    pub fn find_by_vendor(
        &self,
        vendor_id: u16,
        device_id: u16,
    ) -> Option<&PciDevice> {
        self.devices.iter().find(|dev| {
            dev.vendor_id == vendor_id && dev.device_id == device_id
        })
    }
}
