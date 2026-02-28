use core::sync::atomic::{AtomicU64, Ordering};

use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{instructions::interrupts, structures::idt::InterruptStackFrame};

use crate::print;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub static PIC1_OFFSET: u8 = 32;
pub static PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub extern "x86-interrupt" fn spurious_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

pub fn install() {
    interrupts::without_interrupts(|| unsafe {
        PICS.lock().initialize();
    });

    enable();
}

pub fn enable() {
    interrupts::enable();
}

pub fn disable() {
    interrupts::disable();
}
