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

use crate::arch::addr::KVirtAddr;
use crate::arch::generic::paging::MapFlags;
use crate::arch::layout::{KERNEL_STACK_SIZE, PAGE_SIZE};
use crate::system::mem::vmm::AddressSpace;

/// the guard page plus the stack itself.
const RESERVATION_SIZE: usize = KERNEL_STACK_SIZE + PAGE_SIZE;

pub struct KKernelStack {
    base: Option<KVirtAddr>,
    top: KVirtAddr,
    pointer: u64,
}

impl KKernelStack {
    pub const fn placeholder() -> Self {
        Self { base: None, top: KVirtAddr::new(0), pointer: 0 }
    }

    pub fn new() -> Result<Self, &'static str> {
        let base = AddressSpace::reserve_virt(RESERVATION_SIZE)?;
        let bottom = KVirtAddr::new(base.as_u64() + PAGE_SIZE as u64);
        let top = KVirtAddr::new(bottom.as_u64() + KERNEL_STACK_SIZE as u64);

        if let Err(e) = AddressSpace::kernel().map_range_alloc(
            bottom,
            KERNEL_STACK_SIZE,
            MapFlags::READ | MapFlags::WRITE,
        ) {
            let _ = AddressSpace::release_virt(base, RESERVATION_SIZE);
            return Err(e);
        }

        Ok(Self { base: Some(base), top, pointer: 0 })
    }

    pub const fn top(&self) -> KVirtAddr { self.top }

    pub const fn pointer(&self) -> u64 { self.pointer }

    pub fn pointer_mut(&mut self) -> &mut u64 { &mut self.pointer }

    pub fn set_pointer(&mut self, pointer: u64) { self.pointer = pointer; }

    pub const fn is_valid(&self) -> bool {
        self.base.is_some() && self.pointer != 0
    }

    pub const fn is_allocated(&self) -> bool { self.base.is_some() }
}

impl Drop for KKernelStack {
    fn drop(&mut self) {
        let Some(base) = self.base else { return };

        let bottom = KVirtAddr::new(base.as_u64() + PAGE_SIZE as u64);

        if let Err(e) =
            AddressSpace::kernel().unmap_range(bottom, KERNEL_STACK_SIZE)
        {
            log::error!("failed to unmap kernel stack: {}", e);
        }

        if let Err(e) = AddressSpace::release_virt(base, RESERVATION_SIZE) {
            log::error!("failed to release kernel stack reservation: {}", e);
        }
    }
}
