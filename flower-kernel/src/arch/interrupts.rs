use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::instructions::interrupts;
use x86_64::structures::idt::InterruptStackFrame;

use crate::arch::apic;
use crate::system::proc;

static TICKS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Spurious = 255,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 { self as u8 }

    pub fn as_usize(self) -> usize { usize::from(self.as_u8()) }
}

pub fn enable() { interrupts::enable(); }

pub fn disable() { interrupts::disable(); }

pub fn get_ticks() -> u64 { TICKS.load(Ordering::Relaxed) }

pub extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    apic::eoi();
    proc::schedule();
}

pub extern "x86-interrupt" fn spurious_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
}
