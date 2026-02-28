pub mod gdt;
pub mod idt;
pub mod interrupts;

use core::arch::asm;

use crate::info;

pub fn install() {
    gdt::install();
    info!("GDT installed.");
    idt::install();
    info!("IDT installed.");
}

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
