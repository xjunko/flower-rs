use limine::memory_map::EntryType;
use spin::Mutex;
use x86_64::VirtAddr;

use crate::{error, info, system::mem::PAGE_SIZE};

static PMM: Mutex<Option<BitmapAllocator>> = Mutex::new(None);

unsafe impl Send for BitmapAllocator {}
unsafe impl Sync for BitmapAllocator {}
struct BitmapAllocator {
    bitmap: *mut u8,
    bitmap_size: usize,
    total_pages: usize,
    usable_pages: usize,
    free_pages: usize,
}

impl BitmapAllocator {
    fn set_bit(&mut self, bit: usize) {
        let byte_idx = bit / 8;
        let bit_idx = bit % 8;
        unsafe {
            let byte = self.bitmap.add(byte_idx);
            *byte |= 1 << bit_idx;
        }
    }

    fn clear_bit(&mut self, bit: usize) {
        let byte_idx = bit / 8;
        let bit_idx = bit % 8;
        unsafe {
            let byte = self.bitmap.add(byte_idx);
            *byte &= !(1 << bit_idx);
        }
    }

    fn test_bit(&self, bit: usize) -> bool {
        let byte_idx = bit / 8;
        let bit_idx = bit % 8;
        unsafe {
            let byte = *self.bitmap.add(byte_idx);
            byte & (1 << bit_idx) != 0
        }
    }

    fn alloc_page(&mut self) -> Option<u64> {
        for i in 0..self.total_pages {
            if !self.test_bit(i) {
                self.set_bit(i);
                self.free_pages -= 1;
                return Some((i * PAGE_SIZE) as u64);
            }
        }
        None
    }

    fn free_page(&mut self, addr: u64) {
        let page = (addr as usize) / PAGE_SIZE;
        if page < self.total_pages && self.test_bit(page) {
            self.clear_bit(page);
            self.free_pages += 1;
        }
    }
}

fn page_to_mb(page: usize) -> usize {
    (page * PAGE_SIZE) / (1024 * 1024)
}

pub fn install() {
    let (hhdm, mmap) = {
        (
            VirtAddr::new(
                crate::boot::limine::HHDM_REQUEST
                    .get_response()
                    .expect("no hhdm")
                    .offset(),
            ),
            crate::boot::limine::MEMORY_MAP_REQUEST
                .get_response()
                .expect("no mmap")
                .entries(),
        )
    };

    let mut highest_addr: u64 = 0;

    for entry in mmap {
        let end = entry.base + entry.length;
        if end > highest_addr {
            highest_addr = end;
        }
    }

    let total_pages = (highest_addr as usize + PAGE_SIZE - 1) / PAGE_SIZE;
    let bitmap_size = (total_pages + 7) / 8;

    let mut bitmap_addr: Option<u64> = None;
    for entry in mmap {
        if entry.entry_type == EntryType::USABLE && entry.length >= bitmap_size as u64 {
            bitmap_addr = Some(entry.base);
            break;
        }
    }

    let bitmap_addr = bitmap_addr.expect("no space for pmm bitmap");
    let bitmap_ptr = (bitmap_addr + hhdm.as_u64()) as *mut u8;

    // set all bits to 1 (allocated)
    unsafe {
        core::ptr::write_bytes(bitmap_ptr, 0xff, bitmap_size);
    }

    let mut allocator = BitmapAllocator {
        bitmap: bitmap_ptr,
        bitmap_size,
        total_pages,
        usable_pages: 0,
        free_pages: 0,
    };

    for entry in mmap {
        if entry.entry_type == EntryType::USABLE {
            let start_page = (entry.base as usize + PAGE_SIZE - 1) / PAGE_SIZE;
            let end_page = (entry.base + entry.length) as usize / PAGE_SIZE;

            for page in start_page..end_page {
                allocator.clear_bit(page);
                allocator.free_pages += 1;
            }
        }
    }

    let bitmap_start_page = (bitmap_addr as usize) / PAGE_SIZE;
    let bitmap_end_page = ((bitmap_addr as usize) + bitmap_size + PAGE_SIZE - 1) / PAGE_SIZE;

    for page in bitmap_start_page..bitmap_end_page {
        if !allocator.test_bit(page) {
            allocator.set_bit(page);
            allocator.free_pages -= 1;
        }
    }

    allocator.usable_pages = allocator.free_pages;

    info!("PMM installed.");
    info!(
        "PMM: total pages: {}MiB, usable pages: {}MiB, free pages: {}MiB",
        page_to_mb(total_pages),
        page_to_mb(allocator.usable_pages),
        page_to_mb(allocator.free_pages)
    );
    info!(
        "PMM: HHDM: {:#x}, bitmap at {:#x} ({} bytes)",
        hhdm.as_u64(),
        bitmap_addr,
        bitmap_size
    );

    *PMM.lock() = Some(allocator);
}

pub fn alloc() -> Option<u64> {
    if let Some(pmm) = PMM.lock().as_mut() {
        pmm.alloc_page()
    } else {
        None
    }
}

pub fn free(addr: u64) {
    // if address is not aligned, reject it
    if !addr.is_multiple_of(PAGE_SIZE as u64) {
        error!("attempted to free unaligned address: {:#x}", addr);
        return;
    }

    if let Some(pmm) = PMM.lock().as_mut() {
        pmm.free_page(addr);
    }
}

pub fn max_phys_address() -> Option<u64> {
    PMM.lock()
        .as_ref()
        .map(|pmm| (pmm.total_pages * PAGE_SIZE) as u64)
}
