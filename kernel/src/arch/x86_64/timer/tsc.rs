use raw_cpuid::CpuId;

use crate::arch::x86_64::timer::acpi_pmt;

fn __measure_tsc_freq() -> u64 {
    let mut total: u64 = 0;
    for _ in 0..3 {
        unsafe { core::arch::x86_64::_mm_lfence() }
        let start = unsafe { core::arch::x86_64::_rdtsc() };
        acpi_pmt::wait_ms(10);

        unsafe { core::arch::x86_64::_mm_lfence() }
        let end = unsafe { core::arch::x86_64::_rdtsc() };
        total += (end - start) * 100;
    }

    total / 3
}

pub fn measure_tsc_freq() -> u64 {
    let cpuid = CpuId::new();

    // intel only
    if let Some(vendor) = cpuid.get_vendor_info() {
        match vendor.as_str() {
            "GenuineIntel" | "GenuineIotel" => {
                if let Some(tsc_info) = cpuid.get_tsc_info()
                    && let Some(tsc_freq) = tsc_info.tsc_frequency()
                {
                    return tsc_freq;
                }
            },
            _ => {},
        }
    }

    let freq = __measure_tsc_freq();
    log::info!("tsc frequency: {}hz", freq);
    freq
}
