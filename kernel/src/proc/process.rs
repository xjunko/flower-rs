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

use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::Mutex;

use crate::arch::{MapFlags, Multitasking, Paging, Processor};
use crate::memory::vmm::AddressSpace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping(u64),
    Zombie,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessLevel {
    Kernel,
    User,
}

pub struct Process {
    pub(crate) id: Mutex<u64>,
    pub(crate) name: Mutex<String>,
    pub(crate) state: Mutex<ProcessState>,
    pub(crate) level: Mutex<ProcessLevel>,
    pub(crate) address_space: Mutex<Option<AddressSpace>>,

    pub(crate) parent_id: Mutex<Option<u64>>,
    pub(crate) return_code: Mutex<Option<u64>>,

    pub(crate) _stack_ptr: u64,
    pub(crate) _stack_top: u64,
    pub(crate) _stack_bottom: u64,
    pub(crate) _stack_reservation: u64,
}

// kernel proc
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

impl Process {
    pub fn new(name: &str, entry: fn()) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        let stack_size = Processor::KERNEL_STACK_SIZE;
        let reservation_size = stack_size + Processor::PAGE_SIZE;

        let kernel_space = AddressSpace::kernel();
        let stack_virt = AddressSpace::reserve_virt(reservation_size)
            .expect("failed to reserve process stack virtual space");

        let stack_bottom = stack_virt + Processor::PAGE_SIZE as u64;
        let stack_top = stack_bottom + stack_size as u64;

        kernel_space
            .map_range_alloc(
                stack_bottom,
                stack_size,
                MapFlags::READ | MapFlags::WRITE,
            )
            .expect("failed to allocate process stack");

        let stack = Processor::prepare_stack(stack_top, entry);

        Self {
            id: id.into(),
            name: String::from(name).into(),
            state: ProcessState::Ready.into(),
            level: ProcessLevel::Kernel.into(),
            address_space: None.into(),

            parent_id: None.into(),
            return_code: None.into(),

            _stack_ptr: stack,
            _stack_top: stack_top,
            _stack_bottom: stack_bottom,
            _stack_reservation: stack_virt,
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let stack_size = crate::arch::x86_64::layout::PROCESS_STACK_SIZE;
        AddressSpace::kernel()
            .unmap_range(self._stack_bottom, stack_size)
            .expect("failed to free process stack");
    }
}
