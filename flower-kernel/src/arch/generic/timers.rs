use crate::arch::{Processor, Timers};

pub fn install() { Processor::timer_install(); }

pub fn ticks() -> u64 { Processor::timer_get_ticks() }
