pub mod gdt;
pub mod idt;
pub mod interrupts;

use core::arch::asm;

use raw_cpuid::CpuId;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

use crate::info;

fn install_features() {
    let cpuid = CpuId::new();
    if let Some(finfo) = cpuid.get_feature_info() {
        if finfo.has_sse() {
            unsafe {
                Cr0::update(|flags| {
                    flags.remove(Cr0Flags::EMULATE_COPROCESSOR | Cr0Flags::TASK_SWITCHED);
                    flags.insert(Cr0Flags::MONITOR_COPROCESSOR);
                });

                Cr4::update(|flags: &mut Cr4Flags| {
                    flags.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE);
                });
            }
            info!("SSE enabled");
        } else {
            panic!("cpu does not support SSE");
        }
    }
}

pub fn install() {
    install_features();
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
