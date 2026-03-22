use crate::drivers::pci::parser::PciBus;
use crate::info;

mod io;
mod parser;

pub fn install() {
    let mut pci_bus = PciBus::new();
    pci_bus.parse();

    if let Some(ac97) = pci_bus.find_by_class(0x04, 0x01) {
        for bar in ac97.bars.iter().flatten() {
            info!("found ac97 audio device with bar: {:?}", bar);
        }
    }
}
