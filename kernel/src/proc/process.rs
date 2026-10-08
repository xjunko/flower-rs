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
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::Mutex;

use crate::arch::x86_64::layout::PROCESS_STACK_SIZE;
use crate::arch::{Multitasking, Processor};
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

    pub(crate) _stack_arr: Vec<u8>,
    pub(crate) _stack_ptr: u64,
    pub(crate) _stack_top: u64,
    pub(crate) _stack_bottom: u64,
}

// kernel proc
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

impl Process {
    pub fn new(name: &str, entry: fn()) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let stack_arr = alloc::vec![0u8; PROCESS_STACK_SIZE];

        let stack_bottom = stack_arr.as_ptr() as usize;
        let stack_top = stack_bottom + PROCESS_STACK_SIZE;

        let stack = Processor::prepare_stack(stack_top as u64, entry);

        Self {
            id: id.into(),
            name: String::from(name).into(),
            state: ProcessState::Ready.into(),
            level: ProcessLevel::Kernel.into(),
            address_space: None.into(),

            parent_id: None.into(),
            return_code: None.into(),

            _stack_arr: stack_arr,
            _stack_ptr: stack,
            _stack_top: stack_top as u64,
            _stack_bottom: stack_bottom as u64,
        }
    }
}
