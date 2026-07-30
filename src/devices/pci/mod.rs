use crate::devices::pci::parser::PciBus;

pub mod drivers;
mod io;
mod parser;

pub fn install() {
    let mut pci_bus = PciBus::new();
    pci_bus.parse();

    drivers::ac97::install(&pci_bus);
}
