pub mod generic;
pub use self::generic::*;
use crate::arch::addr::{PhysAddr, VirtAddr};
use crate::arch::generic::paging::MapFlags;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

pub trait Port: Copy {
    unsafe fn read(port: u32) -> Self;
    unsafe fn write(port: u32, value: Self);
}

pub trait Arch: Paging + Multitasking {
    // cpu
    fn cpu_bsp_install();
    fn cpu_ap_install();
    fn cpu_halt() -> !;

    // timers
    fn timer_install();
    fn timer_get_ticks() -> u64;

    // ports
    fn port_read<T: Port>(port: u32) -> T;
    fn port_write<T: Port>(port: u32, value: T);

    // stacks
    fn stack_set_kernel(addr: u64);

    // interrupts
    fn interrupts_enable();
    fn interrupts_disable();
    fn interrupts_ack();
    fn interrupts_enabled() -> bool;
    fn interrupts_without<F, R>(f: F) -> R
    where F: FnOnce() -> R {
        let enabled = Self::interrupts_enabled();
        if enabled {
            Self::interrupts_disable();
        }
        let result = f();
        if enabled {
            Self::interrupts_enable();
        }
        result
    }
}

pub trait Paging {
    type Root: Copy + Eq;

    const PAGE_SIZE: usize;
    const KERNEL_HALF_START: u64;

    fn paging_install();

    fn paging_hhdm() -> u64;

    fn paging_root_new() -> Result<Self::Root, &'static str>;
    fn paging_root_free(root: Self::Root);
    fn paging_root_phys(root: Self::Root) -> u64;

    fn paging_root_active() -> Self::Root;
    fn paging_root_set(root: Self::Root);

    fn paging_map(
        root: Self::Root,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str>;

    fn paging_unmap(
        root: Self::Root,
        virt: VirtAddr,
    ) -> Result<PhysAddr, &'static str>;

    fn paging_translate(
        root: Self::Root,
        virt: VirtAddr,
    ) -> Option<(PhysAddr, MapFlags)>;

    fn paging_flush(virt: VirtAddr);
    fn paging_flush_all();
}

pub trait Multitasking: Paging {
    fn prepare_stack_kernel(kernel_stack_top: u64, entry: fn()) -> u64;
    fn prepare_stack_user(
        kernel_stack_top: u64,
        entry: fn(),
        user_stack_top: u64,
    ) -> u64;

    fn switch_context(
        old_sp: *mut u64,
        new_sp: u64,
        new_root: Option<<Self as Paging>::Root>,
    );
}

pub struct Processor {}
