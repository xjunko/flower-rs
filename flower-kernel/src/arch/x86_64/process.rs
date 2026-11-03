use crate::arch::addr::VirtAddr;
use crate::arch::x86_64::gdt;
use crate::arch::{Multitasking, Processor};

impl Multitasking for Processor {
    fn prepare_stack_kernel(kernel_stack_top: u64, entry: fn()) -> u64 {
        todo!()
    }

    fn prepare_stack_user(
        kernel_stack_top: u64,
        entry: fn(),
        user_stack_top: u64,
    ) -> u64 {
        todo!()
    }

    fn stack_set_kernel(addr: VirtAddr) {
        gdt::set_kernel_stack(addr.as_u64());
    }

    fn switch_context(
        old_sp: *mut u64,
        new_sp: u64,
        new_root: Option<<Self as crate::arch::Paging>::Root>,
    ) {
        todo!()
    }
}
