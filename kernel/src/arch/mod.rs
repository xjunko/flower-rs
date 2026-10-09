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

mod generic;
pub use generic::*;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub trait Arch: Paging + Multitasking {
    // cpu
    fn bsp_install();
    fn ap_install();
    fn paging_install();
    fn halt() -> !;

    // timers
    fn timer_install();
    fn timer_get_ns() -> u64;

    // ports
    fn write<T>(p: u32, d: T);
    fn read<T>(p: u32) -> T;

    // stacks
    fn set_kernel_stack(v: u64);

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
    /// opaque handle to a root table (pml4 on x86)
    type Root: Copy + Eq;

    const PAGE_SIZE: usize;
    const KERNEL_HALF_START: u64;

    // setup
    fn hhdm_offset() -> u64;
    fn active_root() -> Self::Root;
    fn set_active_root(root: Self::Root);

    // root lifecycle
    fn root_new() -> Result<Self::Root, &'static str>;
    fn root_free(root: Self::Root);
    fn root_phys(root: Self::Root) -> u64;

    // mapping
    fn map(
        root: Self::Root,
        virt: u64,
        phys: u64,
        flags: MapFlags,
    ) -> Result<(), &'static str>;
    fn unmap(root: Self::Root, virt: u64) -> Result<u64, &'static str>;
    fn protect(
        root: Self::Root,
        virt: u64,
        flags: MapFlags,
    ) -> Result<(), &'static str>;
    fn translate(root: Self::Root, virt: u64) -> Option<(u64, MapFlags)>;

    // tlb
    fn tlb_flush(virt: u64);
    fn tlb_flush_all();

    // fork support
    fn clone_user(root: Self::Root) -> Result<Self::Root, &'static str>;
}

pub trait Multitasking: Paging {
    const KERNEL_STACK_SIZE: usize;

    fn prepare_stack(stack_top: u64, entry: fn()) -> u64;
    fn switch_context(
        old_sp: *mut u64,
        new_sp: u64,
        new_root: Option<<Self as Paging>::Root>,
    );
}

pub struct Processor {}
