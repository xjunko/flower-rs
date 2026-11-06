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

use alloc::format;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::arch::addr::VirtAddr;
use crate::arch::layout::KERNEL_STACK_SIZE;
use crate::arch::paging::MapFlags;
use crate::arch::{self, Multitasking, Paging, Processor};
use crate::system::mem::vmm::AddressSpace;
use crate::system::posix::errno::KResult;
use crate::system::syscalls::SyscallFrame;
use crate::system::vfs::FdTable;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq)]
struct ProcessUserMemory {
    entry: u64,
    stack: u64,
    heap: u64,
    heap_position: u64,
    stack_bottom: u64,
    stack_top: u64,
    heap_top: u64,
    heap_max: u64,
}

impl ProcessUserMemory {
    fn new() -> Self {
        Self {
            entry: 0,
            stack: 0,
            heap: 0,
            heap_position: 0,
            stack_bottom: 0,
            stack_top: 0,
            heap_top: 0,
            heap_max: 0,
        }
    }

    fn from_user(entry: u64, stack: u64, heap: u64) -> Self {
        Self { entry, stack, heap, heap_position: heap, ..Self::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping(u64),
    Zombie,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessLevel {
    RING0,
    RING3,
}

pub struct Process {
    pub id: u64,
    pub name: String,
    pub state: ProcessState,
    pub level: ProcessLevel,
    pub address_space: Option<AddressSpace>,
    pub parent_id: Option<u64>,
    pub exit_status: Option<u64>,
    pub fds: FdTable,

    pub cr3: u64,

    pub stack_ptr: u64,
    pub kernel_stack_top: u64,

    pub _fsbase: u64,
    user: ProcessUserMemory,
}

impl Process {
    pub fn user_entry(&self) -> u64 { self.user.entry }

    pub fn set_user_entry(&mut self, user_entry: u64) {
        self.user.entry = user_entry;
    }

    pub fn user_stack(&self) -> u64 { self.user.stack }

    pub fn set_user_stack(&mut self, user_stack: u64) {
        self.user.stack = user_stack;
    }

    pub fn user_heap(&self) -> u64 { self.user.heap }

    pub fn set_user_heap(&mut self, user_heap: u64) {
        self.user.heap = user_heap;
    }

    pub fn user_heap_position(&self) -> u64 { self.user.heap_position }

    pub fn set_user_heap_position(&mut self, user_heap_position: u64) {
        self.user.heap_position = user_heap_position;
    }

    pub fn user_stack_bottom(&self) -> u64 { self.user.stack_bottom }

    pub fn user_stack_top(&self) -> u64 { self.user.stack_top }

    pub fn set_user_stack_bounds(&mut self, bottom: u64, top: u64) {
        self.user.stack_bottom = bottom;
        self.user.stack_top = top;
    }

    pub fn user_heap_top(&self) -> u64 { self.user.heap_top }

    pub fn user_heap_max(&self) -> u64 { self.user.heap_max }

    pub fn set_user_heap_bounds(&mut self, top: u64, max: u64) {
        self.user.heap_top = top;
        self.user.heap_max = max;
    }
}

impl Process {
    pub fn with_fd_table<F, R>(&mut self, f: F) -> KResult<R>
    where F: FnOnce(&mut FdTable) -> KResult<R> {
        f(&mut self.fds)
    }
}

impl Process {
    pub fn valid_stack(&self) -> bool {
        self.kernel_stack_top != 0 && self.stack_ptr != 0
    }

    pub unsafe fn switch_stack(&self) {
        arch::stacks::set_kernel_stack(VirtAddr::new(self.kernel_stack_top));
        arch::stacks::set_user_stack(VirtAddr::new(self.user_stack()));
        arch::stacks::write_context();
    }
}

#[allow(clippy::fn_to_numeric_cast)]
impl Process {
    /// creates a new kernel process
    pub fn new(name: &str, entry: fn()) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        let stack_size = KERNEL_STACK_SIZE;
        let reservation_size = stack_size + Processor::PAGE_SIZE;

        let stack_virt = AddressSpace::reserve_virt(reservation_size)
            .expect("failed to allocate process stack virt");

        let stack_bottom = stack_virt + Processor::PAGE_SIZE as u64;
        let stack_top = stack_bottom.as_u64() + stack_size as u64;

        AddressSpace::kernel()
            .map_range_alloc(
                stack_bottom,
                stack_size,
                MapFlags::READ | MapFlags::WRITE,
            )
            .expect("failed to allocate process stack");

        let stack = Processor::prepare_stack_kernel(stack_top, entry);

        let cr3 = Processor::paging_root_active();

        Self {
            id,
            name: String::from(name),
            state: ProcessState::Ready,
            level: ProcessLevel::RING0,
            address_space: None,
            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),

            cr3,

            stack_ptr: stack,
            kernel_stack_top: stack_top,

            user: ProcessUserMemory::new(),

            _fsbase: 0,
        }
    }

    /// creates a new user process
    pub fn new_user(
        name: &str,
        address_space: AddressSpace,
        user_entry: u64,
        user_stack: u64,
        user_heap: u64,
    ) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let stack_size = KERNEL_STACK_SIZE;
        let reservation_size = stack_size + Processor::PAGE_SIZE;

        let stack_virt = AddressSpace::reserve_virt(reservation_size)
            .expect("failed to allocate process stack virt");

        let stack_bottom = stack_virt + Processor::PAGE_SIZE as u64;
        let stack_top = stack_bottom.as_u64() + stack_size as u64;

        AddressSpace::kernel()
            .map_range_alloc(
                stack_bottom,
                stack_size,
                MapFlags::READ | MapFlags::WRITE,
            )
            .expect("failed to allocate process stack");

        let stack =
            Processor::prepare_stack_user(stack_top, user_entry, user_stack);

        let cr3 = address_space.root_phys();

        Self {
            id,
            name: String::from(name),
            state: ProcessState::Ready,
            level: ProcessLevel::RING3,
            address_space: Some(address_space),

            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),
            cr3,

            stack_ptr: stack,
            kernel_stack_top: stack_top,

            user: ProcessUserMemory::from_user(
                user_entry, user_stack, user_heap,
            ),

            _fsbase: 0,
        }
    }

    /// creates a new process by copying the current one
    pub fn new_forked(
        parent: &Process,
        address_space: AddressSpace,
        frame: &SyscallFrame,
    ) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let stack_size = KERNEL_STACK_SIZE;
        let reservation_size = stack_size + Processor::PAGE_SIZE;

        let stack_virt = AddressSpace::reserve_virt(reservation_size)
            .expect("failed to allocate process stack virt");

        let stack_bottom = stack_virt + Processor::PAGE_SIZE as u64;
        let stack_top = stack_bottom.as_u64() + stack_size as u64;

        AddressSpace::kernel()
            .map_range_alloc(
                stack_bottom,
                stack_size,
                MapFlags::READ | MapFlags::WRITE,
            )
            .expect("failed to allocate process stack");

        let mut stack_ptr = stack_top;
        let mut child_frame = *frame;
        child_frame.rax = 0;

        unsafe {
            for value in [
                child_frame.ss,
                child_frame.rsp,
                child_frame.rflags,
                child_frame.cs,
                child_frame.rip,
                child_frame.rax as u64,
                child_frame.rcx,
                child_frame.rdx,
                child_frame.rbx,
                child_frame.rbp,
                child_frame.rsi,
                child_frame.rdi,
                child_frame.r8,
                child_frame.r9,
                child_frame.r10,
                child_frame.r11,
                child_frame.r12,
                child_frame.r13,
                child_frame.r14,
                child_frame.r15,
            ] {
                stack_ptr -= 8;
                (stack_ptr as *mut u64).write(value);
            }

            stack_ptr = Processor::prepare_stack_fork(stack_ptr);
        }

        let cr3 = address_space.root_phys();

        Self {
            id,
            name: format!("{}-fork", parent.name),
            state: ProcessState::Ready,
            level: parent.level,
            address_space: Some(address_space),
            parent_id: Some(parent.id),
            exit_status: None,
            fds: parent.fds.clone(),

            cr3,

            stack_ptr,
            kernel_stack_top: stack_top,

            user: ProcessUserMemory {
                entry: frame.rip,
                stack: frame.rsp,
                heap: parent.user_heap(),
                heap_position: parent.user_heap_position(),
                stack_bottom: parent.user_stack_bottom(),
                stack_top: parent.user_stack_top(),
                heap_top: parent.user_heap_top(),
                heap_max: parent.user_heap_max(),
            },

            _fsbase: parent._fsbase,
        }
    }
}

/// creates a null process that does nothing and never sleeps, used as the initial process before the scheduler starts.
pub fn null_process() -> Process {
    let cr3 = Processor::paging_root_active();

    Process {
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        name: String::from("null"),
        state: ProcessState::Running,
        level: ProcessLevel::RING0,
        address_space: None,
        parent_id: None,
        exit_status: None,
        fds: FdTable::new(),

        cr3,

        stack_ptr: 0,
        kernel_stack_top: 0,

        user: ProcessUserMemory::new(),
        _fsbase: 0,
    }
}
