pub const PAGE_SIZE: usize = 4096;

pub const KERNEL_STACK_SIZE: usize = PAGE_SIZE;
pub const PROCESS_STACK_SIZE: usize = PAGE_SIZE * 4;

pub const KERNEL_HEAP_START: usize = 0xFFFF_9000_0000_0000;
pub const KERNEL_HEAP_SIZE: usize = 1024 * 1024;

pub const USER_STACK_TOP_PAGE: u64 = 0x0000_0000_7FFF_F000;
pub const USER_STACK_PAGES: u64 = 256; // 1mb stack
pub const USER_STACK_INITIAL_SLACK: u64 = 0x100;

pub const USER_DYNAMIC_LINKER_BASE: u64 = 0x0000_0000_7FFF_E000;
