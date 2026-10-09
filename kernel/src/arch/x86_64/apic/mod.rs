pub mod ioapic;
pub mod lapic;

use raw_cpuid::CpuId;
use spin::Once;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

use crate::arch::x86_64::apic::ioapic::IoApic;
use crate::arch::x86_64::apic::lapic::LocalApic;
use crate::arch::x86_64::interrupts::InterruptIndex;
use crate::system::mem::vmm::AddressSpace;

// legacy pic
fn pic_disable() {
    const PIC1: u16 = 0x20;
    const PIC1_DATA: u16 = PIC1 + 1;

    const PIC2: u16 = 0xA0;
    const PIC2_DATA: u16 = PIC2 + 1;

    interrupts::without_interrupts(|| {
        let mut p1_data: Port<u8> = Port::new(PIC1_DATA);
        let mut p2_data: Port<u8> = Port::new(PIC2_DATA);

        unsafe {
            p1_data.write(0xFF);
            p2_data.write(0xFF);
        }
    })
}

pub struct Apic {
    pub lapic: LocalApic,
    pub ioapic: IoApic,
}

static APIC: Once<Apic> = Once::new();

pub fn install() {
    pic_disable();

    let cpuid = CpuId::new();
    let address_space = AddressSpace::current();

    if let Some(features) = cpuid.get_feature_info() {
        if !features.has_apic() {
            panic!("cpu does not support apic");
        }

        if features.has_x2apic() {
            log::warn!("x2apic supported but unused, falling back to xapic");
        }
    }

    let lapic = LocalApic::install(&address_space);
    let ioapic = IoApic::install(&address_space);

    lapic.calibrate();
    lapic.enable_spurious_at(InterruptIndex::Spurious as u8);
    lapic.enable_periodic_timer_at(InterruptIndex::LapicTimer as u8);

    // TODO: add back PS/2
    // let acpi_tables = acpi::get();
    // ioapic.set_redirection(
    //     acpi_tables.irq_to_gsi(1),
    //     InterruptIndex::Keyboard as u8,
    //     lapic.id(),
    // );

    // ioapic.set_redirection(
    //     acpi_tables.irq_to_gsi(12),
    //     InterruptIndex::Mouse as u8,
    //     lapic.id(),
    // );

    APIC.call_once(|| Apic { lapic, ioapic });
}

pub fn eoi() { APIC.get().expect("apic not installed").lapic.eoi(); }
