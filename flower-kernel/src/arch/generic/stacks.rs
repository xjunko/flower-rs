use crate::arch::addr::VirtAddr;
use crate::arch::{Multitasking, Processor};

pub fn set_kernel_stack(stack: VirtAddr) { Processor::stack_set_kernel(stack); }
