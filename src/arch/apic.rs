use raw_cpuid::CpuId;
use x86_64::{
    PhysAddr, VirtAddr,
    instructions::{interrupts, port::Port},
    registers::model_specific::{ApicBase, ApicBaseFlags},
    structures::paging::PageTableFlags,
};

use crate::{arch::interrupts::InterruptIndex, debug, error, system::mem::vmm};

// legacy pic
const PIC1: u16 = 0x20;
const PIC1_COMMAND: u16 = PIC1;
const PIC1_DATA: u16 = PIC1 + 1;

const PIC2: u16 = 0xA0;
const PIC2_COMMAND: u16 = PIC2;
const PIC2_DATA: u16 = PIC2 + 1;

fn pic_disable() {
    interrupts::without_interrupts(|| {
        let mut p1_data: Port<u8> = Port::new(PIC1_DATA);
        let mut p2_data: Port<u8> = Port::new(PIC2_DATA);

        unsafe {
            p1_data.write(0xFF);
            p2_data.write(0xFF);
        }
    })
}

// lapic
const LAPIC_VIRT: u64 = 0xFFFF_FFFF_FEE0_0000;

pub const LAPIC_EOI: u64 = 0x0B0;
pub const LAPIC_SPURIOUS: u64 = 0x0F0;
pub const LAPIC_TIMER_LVT: u64 = 0x320;
pub const LAPIC_TIMER_INIT: u64 = 0x380;
pub const LAPIC_TIMER_CURRENT: u64 = 0x390;
pub const LAPIC_TIMER_DIV: u64 = 0x3E0;

const IOAPIC_VIRT: u64 = 0xFFFF_FFFF_FEC0_0000;
const IOAPIC_REG_SELECT: u64 = 0x00;
const IOAPIC_REG_DATA: u64 = 0x10;

pub unsafe fn lapic_read(offset: u64) -> u32 {
    let ptr = (LAPIC_VIRT + offset) as *const u32;
    unsafe { core::ptr::read_volatile(ptr) }
}

pub unsafe fn lapic_write(offset: u64, value: u32) {
    let ptr = (LAPIC_VIRT + offset) as *mut u32;
    unsafe { core::ptr::write_volatile(ptr, value) }
}

pub unsafe fn ioapic_read(reg: u32) -> u32 {
    let select = IOAPIC_VIRT as *mut u32;
    let data = (IOAPIC_VIRT + IOAPIC_REG_DATA) as *mut u32;
    unsafe { core::ptr::write_volatile(select, reg) };
    unsafe { core::ptr::read_volatile(data) }
}

pub unsafe fn ioapic_write(reg: u32, value: u32) {
    let select = IOAPIC_VIRT as *mut u32;
    let data = (IOAPIC_VIRT + IOAPIC_REG_DATA) as *mut u32;
    unsafe { core::ptr::write_volatile(select, reg) };
    unsafe { core::ptr::write_volatile(data, value) };
}

const PIT_FREQ: u32 = 1193182;
const CALIBRATE_MS: u32 = 10;

static mut TICKS_PER_MS: u32 = 0;

fn calibrate_timer() {
    unsafe {
        let mut pit_cmd: Port<u8> = Port::new(0x43);
        let mut pit_ch2: Port<u8> = Port::new(0x42);
        let mut pit_gate: Port<u8> = Port::new(0x61);

        let divisor = (PIT_FREQ / 1000) * CALIBRATE_MS;

        pit_cmd.write(0b10110010);

        let gate = pit_gate.read();
        pit_gate.write(gate | 0x01);

        pit_ch2.write((divisor & 0xFF) as u8);
        pit_ch2.write((divisor >> 8) as u8);

        lapic_write(LAPIC_TIMER_DIV, 0x3);
        lapic_write(LAPIC_TIMER_INIT, 0xFFFFFFFF);

        while pit_gate.read() & 0x20 == 0 {}

        let elapsed = 0xFFFFFFFF - lapic_read(LAPIC_TIMER_CURRENT);

        lapic_write(LAPIC_TIMER_INIT, 0);

        TICKS_PER_MS = elapsed / CALIBRATE_MS;
    }
}

pub fn install() {
    // disable the legacy pic
    pic_disable();

    // check if we can even use it
    let cpuid = CpuId::new();
    if let Some(finfo) = cpuid.get_feature_info() {
        // check for x2apic
        if finfo.has_x2apic() {
            error!(
                "x2apic is supported, but kernel doesn't know what to do with it yet, going with APIC"
            );
        }

        if !finfo.has_apic() {
            panic!("cpu does not support apic");
        }

        // get apic base
        let (apic_base, apic_flags) = ApicBase::read();
        debug!("apic addr: {:#x}", apic_base.start_address().as_u64());

        if !apic_flags.contains(ApicBaseFlags::LAPIC_ENABLE) {
            debug!("lapic not enabled, enabling...");
            unsafe {
                ApicBase::write(apic_base, apic_flags | ApicBaseFlags::LAPIC_ENABLE);
            }
            debug!("enabled!");
        } else {
            debug!("lapic already enabled.");
        }

        // map it
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;
        vmm::page_map(VirtAddr::new(LAPIC_VIRT), apic_base.start_address(), flags)
            .expect("failed to map lapic.");

        // also map ioapic
        let acpi_tables = super::acpi::get();
        if acpi_tables.ioapics.is_empty() {
            panic!("no ioapic found in acpi tables");
        }
        let ioapic_addr = acpi_tables.ioapics[0].address;
        debug!("ioapic addr: {:#x}", ioapic_addr);
        vmm::page_map(
            VirtAddr::new(IOAPIC_VIRT),
            PhysAddr::new(ioapic_addr as u64),
            flags,
        )
        .expect("failed to map ioapic");

        // enable spurious
        unsafe {
            lapic_write(LAPIC_SPURIOUS, 0x100 | InterruptIndex::Spurious as u32);
        }

        calibrate_timer();

        // finish
        let ticks_10ms = unsafe { TICKS_PER_MS * 10 };
        unsafe {
            lapic_write(LAPIC_TIMER_DIV, 0x3);
            lapic_write(LAPIC_TIMER_LVT, (1 << 17) | InterruptIndex::Timer as u32);
            lapic_write(LAPIC_TIMER_INIT, ticks_10ms);
        }
    }
}

pub fn eoi() {
    unsafe { lapic_write(LAPIC_EOI, 0) }
}
