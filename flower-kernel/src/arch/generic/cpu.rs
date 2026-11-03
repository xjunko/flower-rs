use crate::arch::{Cpu, Processor};

pub fn bsp_install() { Processor::cpu_bsp_install(); }

pub fn ap_install() { Processor::cpu_ap_install(); }

pub fn halt() -> ! { Processor::cpu_halt() }
