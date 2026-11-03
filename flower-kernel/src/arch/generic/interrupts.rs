use crate::arch::{Interrupts, Processor};

pub fn enable() { Processor::interrupts_enable(); }

pub fn disable() { Processor::interrupts_disable(); }

pub fn ack() { Processor::interrupts_ack(); }

pub fn enabled() -> bool { Processor::interrupts_enabled() }
