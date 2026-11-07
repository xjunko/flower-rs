/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

pub mod generic;
pub use self::generic::*;
use crate::arch::addr::{KPhysAddr, KVirtAddr};
use crate::arch::generic::paging::MapFlags;
use crate::system::posix::errno::KResult;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub trait Port: Copy {
    unsafe fn read(port: u32) -> Self;
    unsafe fn write(port: u32, value: Self);
}

pub trait Arch: Cpu + Paging + Multitasking + Interrupts {
    fn port_read<T: Port>(port: u32) -> T;
    fn port_write<T: Port>(port: u32, value: T);
}

pub trait Cpu {
    fn cpu_bsp_install();
    fn cpu_ap_install();
    fn cpu_halt() -> !;
}

pub trait Timers {
    fn timer_install();
    fn timer_get_ticks() -> u64;
    fn timer_get_ns() -> u64;
}

pub trait Interrupts {
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
    const PAGE_BITS: usize;
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
        virt: KVirtAddr,
        phys: KPhysAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str>;

    fn paging_unmap(
        root: Self::Root,
        virt: KVirtAddr,
    ) -> Result<KPhysAddr, &'static str>;

    fn paging_update(
        root: Self::Root,
        virt: KVirtAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str>;

    fn paging_translate(
        root: Self::Root,
        virt: KVirtAddr,
    ) -> Option<(KPhysAddr, MapFlags)>;

    fn paging_flush(virt: KVirtAddr);
    fn paging_flush_all();

    fn paging_clone_user(root: Self::Root) -> Result<Self::Root, &'static str>;
}

pub trait Multitasking: Paging {
    fn prepare_stack_kernel(kernel_stack_top: u64, entry: fn()) -> u64;
    fn prepare_stack_user(
        kernel_stack_top: u64,
        entry: u64,
        user_stack_top: u64,
    ) -> u64;
    fn prepare_stack_fork(kernel_stack_top: u64) -> u64;

    fn stack_set_kernel(addr: KVirtAddr);
    fn stack_set_user(addr: KVirtAddr);
    fn stack_write_context();

    fn switch_context(
        old_sp: *mut u64,
        new_sp: u64,
        new_root: Option<<Self as Paging>::Root>,
    );
}

pub trait Userspace {
    fn copy_to_user<T: Copy>(src: KVirtAddr, data: T) -> KResult<()>;
    fn copy_from_user<T: Copy>(dest: KVirtAddr) -> KResult<T>;
}

pub struct Processor {}
