use core::panic;

use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::system::mem::PAGE_SIZE;
use crate::system::{self};
use crate::{error, info};

const HEAP_START: u64 = 0xFFFF_9000_0000_0000;
const HEAP_SIZE: usize = 4 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    error!("allocation error in process: {}", system::proc::name());
    error!("requested: {:?}", layout);
    panic!("allocation error");
}

pub fn install() -> Result<(), &'static str> {
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_EXECUTE;
    let heap_pages = (HEAP_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;

    for i in 0..heap_pages {
        let addr = VirtAddr::new(HEAP_START + (i * PAGE_SIZE) as u64);
        system::mem::vmm::page_map_alloc(addr, flags)?;
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    info!(
        "heap installed at {:#x} with size {} MiB",
        HEAP_START,
        HEAP_SIZE / (1024 * 1024)
    );
    Ok(())
}
