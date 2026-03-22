use crate::drivers::pci::parser::PciBus;

pub mod devices;
mod io;
mod parser;

pub fn install() {
    let mut pci_bus = PciBus::new();
    pci_bus.parse();

    devices::ac97::install(&pci_bus);
}
