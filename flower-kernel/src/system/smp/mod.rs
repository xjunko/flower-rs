use limine::mp::Cpu;

use crate::arch;
use crate::boot::limine::SMP_REQUEST;

unsafe extern "C" fn __smp_entry(ap: &Cpu) -> ! {
    log::info!("SMP: core {} started.", ap.lapic_id);
    // we dont actually use them yet.
    loop {
        arch::interrupts::disable();
        arch::halt();
    }
}

pub fn install() {
    if let Some(smp) = SMP_REQUEST.get_response() {
        let cpus = smp.cpus();

        log::info!(
            "SMP: found {} cores, BSP is {}.",
            cpus.len(),
            smp.bsp_lapic_id()
        );

        for cpu in cpus {
            if cpu.lapic_id == smp.bsp_lapic_id() {
                continue; // dont want to mess with this one
            }
            cpu.goto_address.write(__smp_entry);
        }
    } else {
        log::error!("SMP: not supported, not good.");
    }
}
