use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;

use crate::system::mem::vmm::AddressSpace;
use crate::system::proc::trampoline;
use crate::system::syscalls::SyscallFrame;
use crate::system::vfs::{FdTable, VFSResult};
use crate::{arch, system};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping,
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
    pub wake_at: Option<u64>,
    pub parent_id: Option<u64>,
    pub exit_status: Option<u64>,
    pub fds: FdTable,

    pub cr3: u64,

    pub stack_ptr: u64,
    pub kernel_stack_top: u64,

    pub user_entry: u64,
    pub user_stack: u64,

    pub user_heap: u64,
    pub user_heap_position: u64,

    pub _fsbase: u64,
    _stack: Vec<u8>,
}

impl Process {
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
        system::syscalls::set_user_stack(self.user_stack);
        system::syscalls::write_cpu_context();
        arch::gdt::set_kernel_stack(VirtAddr::new(self.kernel_stack_top));
    }
}

#[allow(clippy::fn_to_numeric_cast)]
impl Process {
    const STACK_SIZE: usize = 4096 * 4;

    /// creates a new kernel process
    pub fn new(name: &str, entry: fn()) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let stack = alloc::vec![0u8; Self::STACK_SIZE];

        let stack_top = stack.as_ptr() as u64 + Self::STACK_SIZE as u64;
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
            name: String::from(name),
            state: ProcessState::Ready,
            level: ProcessLevel::RING0,
            address_space: None,
            wake_at: None,
            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),

            cr3: pml4_frame.start_address().as_u64(),

            stack_ptr,
            kernel_stack_top: stack_top,

            user_entry: 0,
            user_stack: 0,

            user_heap: 0,
            user_heap_position: 0,

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
        let stack = alloc::vec![0u8; Self::STACK_SIZE];

        let stack_top = stack.as_ptr() as u64 + Self::STACK_SIZE as u64;
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
            name: String::from(name),
            state: ProcessState::Ready,
            level: ProcessLevel::RING3,
            address_space: Some(address_space),
            wake_at: None,
            parent_id: None,
            exit_status: None,
            fds: FdTable::new(),
            cr3,

            stack_ptr,
            kernel_stack_top: stack_top,

            user_entry,
            user_stack,

            user_heap,
            user_heap_position: user_heap,

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
        let stack = alloc::vec![0u8; Self::STACK_SIZE];

        let stack_top = stack.as_ptr() as u64 + Self::STACK_SIZE as u64;
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
            wake_at: None,
            parent_id: Some(parent.id),
            exit_status: None,
            fds: parent.fds.clone(),

            cr3,

            stack_ptr,
            kernel_stack_top: stack_top,

            user_entry: frame.rip,
            user_stack: frame.rsp,

            user_heap: parent.user_heap,
            user_heap_position: parent.user_heap_position,

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
        name: String::from("null"),
        state: ProcessState::Running,
        level: ProcessLevel::RING0,
        address_space: None,
        wake_at: None,
        parent_id: None,
        exit_status: None,
        fds: FdTable::new(),

        cr3: pml4_frame.start_address().as_u64(),

        stack_ptr: 0,
        kernel_stack_top: 0,

        user_entry: 0,
        user_stack: 0,

        user_heap: 0,
        user_heap_position: 0,

        _fsbase: 0,
        _stack: Vec::new(),
    }
}
