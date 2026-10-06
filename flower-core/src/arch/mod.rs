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

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub trait Arch {
    const PAGE_SIZE: usize;

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
}

pub struct Processor {}
