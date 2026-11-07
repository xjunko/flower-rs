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

use crate::arch::{self, Multitasking, Processor};
use crate::system::mem::kstack::KKernelStack;
use crate::system::mem::vmm::AddressSpace;
use crate::system::posix::errno::KResult;
use crate::system::proc::memory::{KImageLoad, KUserHeap, KUserImage};
use crate::system::syscalls::SyscallFrame;
use crate::system::vfs::FdTable;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping(u64),
    Zombie,
    Dead,
}

/// a schedulable process.
///
/// a process owns its address space, its kernel stack and its heap
/// bookkeeping. a kernel process has no address space and no heap; user
/// process state is only created from a loaded [`KUserImage`].
pub struct Process {
    id: u64,
    name: String,
    state: ProcessState,
    parent_id: Option<u64>,
    exit_status: Option<u64>,

    pub fds: FdTable,

    address_space: Option<AddressSpace>,
    heap: Option<KUserHeap>,
    kernel_stack: KKernelStack,
    fsbase: u64,
}

impl Process {
    pub const fn id(&self) -> u64 { self.id }

    pub fn name(&self) -> &str { &self.name }

    pub fn set_name(&mut self, name: String) { self.name = name; }

    pub const fn state(&self) -> ProcessState { self.state }

    pub fn set_state(&mut self, state: ProcessState) { self.state = state; }

    pub const fn parent_id(&self) -> Option<u64> { self.parent_id }

    pub const fn exit_status(&self) -> Option<u64> { self.exit_status }

    /// true for processes that have a user address space.
    pub const fn is_user(&self) -> bool { self.address_space.is_some() }

    pub fn address_space(&self) -> Option<&AddressSpace> {
        self.address_space.as_ref()
    }

    pub fn address_space_mut(&mut self) -> Option<&mut AddressSpace> {
        self.address_space.as_mut()
    }

    /// the root page table this process runs on. kernel processes run in the
    /// kernel address space.
    pub fn address_space_root(&self) -> u64 {
        match &self.address_space {
            Some(address_space) => address_space.root_phys(),
            None => AddressSpace::kernel().root_phys(),
        }
    }

    pub fn kernel_stack(&self) -> &KKernelStack { &self.kernel_stack }

    pub fn kernel_stack_mut(&mut self) -> &mut KKernelStack {
        &mut self.kernel_stack
    }

    /// true if this process has an allocated kernel stack with a saved
    /// context to switch to.
    pub fn valid_stack(&self) -> bool { self.kernel_stack.is_valid() }

    pub fn heap(&self) -> Option<&KUserHeap> { self.heap.as_ref() }

    pub fn heap_mut(&mut self) -> Option<&mut KUserHeap> { self.heap.as_mut() }

    /// installs a new user image, replacing the address space and resetting
    /// the heap and fsbase.
    ///
    /// the previous address space is returned instead of dropped: the caller
    /// must switch cr3 before dropping it.
    pub fn load_image(&mut self, image: KUserImage) -> KImageLoad {
        let KUserImage {
            address_space,
            entry,
            stack_pointer,
            heap,
            program_break,
        } = image;

        let previous_address_space = self.address_space.replace(address_space);
        self.heap = Some(KUserHeap::new(heap, program_break));
        self.fsbase = 0;

        KImageLoad { previous_address_space, entry, stack_pointer }
    }

    pub const fn fsbase(&self) -> u64 { self.fsbase }

    pub fn set_fsbase(&mut self, fsbase: u64) { self.fsbase = fsbase; }

    /// records the exit status and moves the process to its final state.
    pub fn mark_exited(&mut self, status: u64) {
        self.exit_status = Some(status);
        self.state = if self.parent_id.is_some() {
            ProcessState::Zombie
        } else {
            ProcessState::Dead
        };
    }

    pub fn with_fd_table<F, R>(&mut self, f: F) -> KResult<R>
    where F: FnOnce(&mut FdTable) -> KResult<R> {
        f(&mut self.fds)
    }

    pub unsafe fn switch_stack(&self) {
        arch::stacks::set_kernel_stack(self.kernel_stack.top());
        arch::stacks::write_context();
    }
}

// constructors
impl Process {
    /// creates a kernel process with the given entry point.
    pub fn new_kernel(name: &str, entry: fn()) -> Self {
        let mut kernel_stack = KKernelStack::new()
            .expect("failed to allocate the process kernel stack");
        let pointer =
            Processor::prepare_stack_kernel(kernel_stack.top().as_u64(), entry);
        kernel_stack.set_pointer(pointer);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            name: String::from(name),
            state: ProcessState::Ready,
            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),
            address_space: None,
            heap: None,
            kernel_stack,
            fsbase: 0,
        }
    }

    /// creates a user process from a loaded image.
    pub fn new_user(name: &str, image: KUserImage) -> Self {
        let KUserImage {
            address_space,
            entry,
            stack_pointer,
            heap,
            program_break,
        } = image;

        let mut kernel_stack = KKernelStack::new()
            .expect("failed to allocate the process kernel stack");
        let pointer = Processor::prepare_stack_user(
            kernel_stack.top().as_u64(),
            entry,
            stack_pointer,
        );
        kernel_stack.set_pointer(pointer);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            name: String::from(name),
            state: ProcessState::Ready,
            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),
            address_space: Some(address_space),
            heap: Some(KUserHeap::new(heap, program_break)),
            kernel_stack,
            fsbase: 0,
        }
    }

    /// creates a child process by copying the parent's user-visible state.
    /// the caller supplies the already-cloned address space.
    pub fn new_forked(
        parent: &Process,
        address_space: AddressSpace,
        frame: &SyscallFrame,
    ) -> Self {
        let mut kernel_stack = KKernelStack::new()
            .expect("failed to allocate the process kernel stack");
        let mut stack_ptr = kernel_stack.top().as_u64();

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
        kernel_stack.set_pointer(stack_ptr);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            name: format!("{}-fork", parent.name()),
            state: ProcessState::Ready,
            parent_id: Some(parent.id()),
            exit_status: None,
            fds: parent.fds.clone(),
            address_space: Some(address_space),
            heap: parent.heap().copied(),
            kernel_stack,
            fsbase: parent.fsbase(),
        }
    }

    /// the idle process used before the scheduler starts. it never owns a
    /// stack or an address space.
    pub fn null() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            name: String::from("null"),
            state: ProcessState::Running,
            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),
            address_space: None,
            heap: None,
            kernel_stack: KKernelStack::placeholder(),
            fsbase: 0,
        }
    }
}
