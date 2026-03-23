pub mod acpi;
pub mod apic;
pub mod gdt;
pub mod idt;
pub mod interrupts;

use core::arch::asm;

use raw_cpuid::CpuId;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

pub fn install_cpu_features() {
    let cpuid = CpuId::new();
    if let Some(finfo) = cpuid.get_feature_info() {
        assert!(finfo.has_fxsave_fxstor(), "FXSAVE/FXSTOR not supported");
        assert!(finfo.has_mmx(), "MMX not supported");
        assert!(finfo.has_sse(), "SSE not supported");
        assert!(finfo.has_fpu(), "FPU not supported");

        unsafe {
            Cr0::update(|flags| {
                flags.remove(
                    Cr0Flags::EMULATE_COPROCESSOR | Cr0Flags::TASK_SWITCHED,
                );
                flags.insert(Cr0Flags::MONITOR_COPROCESSOR);
            });

            Cr4::update(|flags: &mut Cr4Flags| {
                flags.insert(Cr4Flags::OSFXSR | Cr4Flags::OSXMMEXCPT_ENABLE);
            });
        }
        log::debug!("SSE enabled");
    }
}

pub fn ticks() -> u64 { interrupts::get_ticks() }

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
