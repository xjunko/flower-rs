use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::{instructions::interrupts, structures::idt::InterruptStackFrame};

use crate::print;

static TICKS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Spurious = 255,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn install() {
    interrupts::enable();
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    print!(".");

    super::apic::eoi();
}

pub extern "x86-interrupt" fn spurious_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    print!(".");

    super::apic::eoi();
}
