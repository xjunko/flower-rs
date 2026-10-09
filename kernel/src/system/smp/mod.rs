use limine::mp::MpInfo;

use crate::arch;
use crate::boot::limine::SMP_REQUEST;

unsafe extern "C" fn __smp_entry(ap: &MpInfo) -> ! {
    log::info!("SMP: core {} started.", ap.lapic_id);
    arch::interrupts::disable();
    arch::halt();
}

pub fn install() {
    if let Some(smp) = SMP_REQUEST.response() {
        let cpus = smp.cpus();

        log::info!(
            "SMP: found {} cores, BSP is {}.",
            cpus.len(),
            smp.bsp_lapic_id
        );

        for cpu in cpus {
            if cpu.lapic_id == smp.bsp_lapic_id {
                continue; // dont want to mess with this one
            }
            cpu.bootstrap(__smp_entry, 0);
        }
    } else {
        log::error!("SMP: not supported, not good.");
    }
}
