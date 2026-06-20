use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;

use crate::arch;
use crate::system::mem::vmm::AddressSpace;
use crate::system::proc::trampoline;
use crate::system::vfs::{FdTable, VFSResult};

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
