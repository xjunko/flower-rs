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
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;

use crate::arch::x86_64::layout::PROCESS_STACK_SIZE;
use crate::memory::vmm::AddressSpace;
use crate::system::proc::trampoline;
use crate::system::syscalls::SyscallFrame;
use crate::system::vfs::{FdTable, VFSResult};
use crate::{arch, system};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq)]
struct UserMemory {
    entry: u64,
    stack: u64,
    heap: u64,
    heap_position: u64,
    stack_bottom: u64,
    stack_top: u64,
    heap_top: u64,
    heap_max: u64,
}

impl UserMemory {
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
    user: UserMemory,
    _stack: Vec<u8>,
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

    pub fn with_fd_table<F, R>(&mut self, f: F) -> VFSResult<R>
    where F: FnOnce(&mut FdTable) -> VFSResult<R> {
        f(&mut self.fds)
    }
}

impl Process {
    pub fn valid_stack(&self) -> bool {
        self.kernel_stack_top != 0 && self.stack_ptr != 0
    }

    pub unsafe fn switch_stack(&self) {
        system::syscalls::set_kernel_stack(self.kernel_stack_top);
        system::syscalls::set_user_stack(self.user_stack());
        system::syscalls::write_cpu_context();
        arch::x86_64::gdt::set_kernel_stack(VirtAddr::new(
            self.kernel_stack_top,
        ));
    }
}

#[allow(clippy::fn_to_numeric_cast)]
impl Process {
    /// creates a new kernel process
    pub fn new(name: &str, entry: fn()) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let stack = alloc::vec![0u8; PROCESS_STACK_SIZE];

        let stack_top = stack.as_ptr() as u64 + PROCESS_STACK_SIZE as u64;
        let stack_top = stack_top & !0xF;

        let mut stack_ptr = stack_top;

        unsafe {
            stack_ptr -= 8;
            (stack_ptr as *mut u64)
                .write(trampoline::kernel_trampoline_entry as *const () as u64);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(entry as u64);
        }

        let (pml4_frame, _) = Cr3::read();

        Self {
            id,
            name: name.to_string(),
            state: ProcessState::Ready,
            level: ProcessLevel::RING0,
            address_space: None,
            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),

            cr3: pml4_frame.start_address().as_u64(),

            stack_ptr,
            kernel_stack_top: stack_top,

            user: UserMemory::new(),

            _fsbase: 0,
            _stack: stack,
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
        let stack = alloc::vec![0u8; PROCESS_STACK_SIZE];

        let stack_top = stack.as_ptr() as u64 + PROCESS_STACK_SIZE as u64;
        let stack_top = stack_top & !0xF; // align to 16 bytes

        let mut stack_ptr = stack_top;

        unsafe {
            stack_ptr -= 8;
            (stack_ptr as *mut u64)
                .write(trampoline::user_trampoline_entry as *const () as u64);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(user_stack);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(user_entry);
        }

        let cr3 = address_space.cr3();

        Self {
            id,
            name: name.to_string(),
            state: ProcessState::Ready,
            level: ProcessLevel::RING3,
            address_space: Some(address_space),

            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),
            cr3,

            stack_ptr,
            kernel_stack_top: stack_top,

            user: UserMemory::from_user(user_entry, user_stack, user_heap),

            _fsbase: 0,
            _stack: stack,
        }
    }

    /// creates a new process by copying the current one
    pub fn new_forked(
        parent: &Process,
        address_space: AddressSpace,
        frame: &SyscallFrame,
    ) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let stack = alloc::vec![0u8; PROCESS_STACK_SIZE];

        let stack_top = stack.as_ptr() as u64 + PROCESS_STACK_SIZE as u64;
        let stack_top = stack_top & !0xF;

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

            stack_ptr -= 8;
            (stack_ptr as *mut u64)
                .write(trampoline::fork_return_trampoline as *const () as u64);

            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);
            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);
            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);
            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);
            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);
            stack_ptr -= 8;
            (stack_ptr as *mut u64).write(0);
        }

        let cr3 = address_space.cr3();

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

            user: UserMemory {
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
            _stack: stack,
        }
    }
}

/// creates a null process that does nothing and never sleeps, used as the initial process before the scheduler starts.
pub fn null_process() -> Process {
    let (pml4_frame, _) = Cr3::read();

    Process {
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        name: "null".to_string(),
        state: ProcessState::Running,
        level: ProcessLevel::RING0,
        address_space: None,
        parent_id: None,
        exit_status: None,
        fds: FdTable::new(),

        cr3: pml4_frame.start_address().as_u64(),

        stack_ptr: 0,
        kernel_stack_top: 0,

        user: UserMemory::new(),
        _fsbase: 0,
        _stack: Vec::new(),
    }
}
