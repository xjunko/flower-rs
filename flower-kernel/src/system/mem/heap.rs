use core::panic;

use linked_list_allocator::LockedHeap;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::system::mem::PAGE_SIZE;
use crate::system::{self};
use crate::{error, info};

const HEAP_START: u64 = 0xFFFF_9000_0000_0000;
const HEAP_DEFAULT_SIZE: usize = 128 * 1024 * 1024;
const HEAP_MIN_SIZE: usize = 16 * 1024 * 1024;
const HEAP_MAX_SIZE: usize = 512 * 1024 * 1024;
const HEAP_INITIAL_MAX_SIZE: usize = 256 * 1024 * 1024;
const HEAP_WINDOW_MAX_SIZE: usize = 2 * 1024 * 1024 * 1024;
const HEAP_WINDOW_FACTOR: usize = 4;
const HEAP_RATIO_DIVISOR: usize = 8;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static HEAP_STATE: Mutex<HeapState> = Mutex::new(HeapState::new());

struct HeapState {
    start: u64,
    mapped_end: u64,
    window_end: u64,
    installed: bool,
}

impl HeapState {
    const fn new() -> Self {
        Self { start: 0, mapped_end: 0, window_end: 0, installed: false }
    }
}

#[alloc_error_handler]
pub fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    error!("allocation error in process: {}", system::proc::name());
    error!("requested: {:?}", layout);
    panic!("allocation error");
}

fn heap_size_from_ram() -> usize {
    let free_bytes = system::mem::pmm::free_pages()
        .unwrap_or(HEAP_DEFAULT_SIZE / PAGE_SIZE)
        .saturating_mul(PAGE_SIZE);

    let mut heap_size =
        (free_bytes / HEAP_RATIO_DIVISOR).clamp(HEAP_MIN_SIZE, HEAP_MAX_SIZE);

    let hard_cap = free_bytes / 2;
    if hard_cap >= PAGE_SIZE {
        heap_size = heap_size.min(hard_cap);
    }

    (heap_size + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE
}

fn heap_initial_size_from_ram() -> usize {
    heap_size_from_ram().min(HEAP_INITIAL_MAX_SIZE)
}

fn heap_window_size_from_ram(initial_size: usize) -> usize {
    let mut window_size = initial_size.saturating_mul(HEAP_WINDOW_FACTOR);
    window_size = window_size.clamp(initial_size, HEAP_WINDOW_MAX_SIZE);
    (window_size + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE
}

fn align_down_to_page(addr: u64) -> u64 {
    let page_mask = (PAGE_SIZE as u64) - 1;
    addr & !page_mask
}

fn align_up_to_page(addr: u64) -> u64 {
    let page_mask = (PAGE_SIZE as u64) - 1;
    (addr + page_mask) & !page_mask
}

pub fn handle_page_fault(addr: VirtAddr) -> bool {
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_EXECUTE;

    let fault_addr = addr.as_u64();
    let mut state = HEAP_STATE.lock();
    if !state.installed
        || fault_addr < state.start
        || fault_addr >= state.window_end
    {
        return false;
    }

    let fault_page = align_down_to_page(fault_addr);
    if fault_page < state.mapped_end {
        return false;
    }

    let target_end = align_up_to_page(fault_addr.saturating_add(1));
    while state.mapped_end < target_end {
        if state.mapped_end >= state.window_end {
            return false;
        }

        let map_addr = VirtAddr::new(state.mapped_end);
        if system::mem::vmm::page_map_alloc(map_addr, flags).is_err() {
            return false;
        }

        state.mapped_end = state.mapped_end.saturating_add(PAGE_SIZE as u64);
    }

    true
}

pub fn install() -> Result<(), &'static str> {
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_EXECUTE;
    let heap_initial_size = heap_initial_size_from_ram();
    let heap_window_size = heap_window_size_from_ram(heap_initial_size);
    let free_mib_before = system::mem::pmm::free_pages()
        .map(|p| (p * PAGE_SIZE) / (1024 * 1024))
        .unwrap_or(0);
    let heap_pages = (heap_initial_size + PAGE_SIZE - 1) / PAGE_SIZE;

    for i in 0..heap_pages {
        let addr = VirtAddr::new(HEAP_START + (i * PAGE_SIZE) as u64);
        system::mem::vmm::page_map_alloc(addr, flags)?;
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, heap_window_size);
    }

    {
        let mut state = HEAP_STATE.lock();
        state.start = HEAP_START;
        state.mapped_end = HEAP_START + heap_initial_size as u64;
        state.window_end = HEAP_START + heap_window_size as u64;
        state.installed = true;
    }

    info!(
        "heap installed at {:#x}: mapped {} MiB, window {} MiB (free RAM before init: {} MiB)",
        HEAP_START,
        heap_initial_size / (1024 * 1024),
        heap_window_size / (1024 * 1024),
        free_mib_before
    );
    Ok(())
}
