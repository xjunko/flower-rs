use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::instructions::interrupts;
use x86_64::structures::idt::InterruptStackFrame;

use crate::arch::{Interrupts, Processor, Timers};
use crate::system::proc;

static TICKS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Keyboard = 33,
    Spurious = 255,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 { self as u8 }

    pub fn as_usize(self) -> usize { usize::from(self.as_u8()) }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    Processor::interrupts_ack();
    proc::schedule();
}

pub extern "x86-interrupt" fn spurious_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    Processor::interrupts_ack();
    todo!()
}

impl Timers for Processor {
    fn timer_install() { crate::arch::x86_64::apic::install(); }

    fn timer_get_ticks() -> u64 { TICKS.load(Ordering::Relaxed) }
}

impl Interrupts for Processor {
    fn interrupts_enable() { interrupts::enable(); }

    fn interrupts_disable() { interrupts::disable(); }

    fn interrupts_ack() { crate::arch::x86_64::apic::eoi(); }

    fn interrupts_enabled() -> bool { interrupts::are_enabled() }
}
