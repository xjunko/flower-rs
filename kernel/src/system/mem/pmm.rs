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

use spin::mutex::SpinMutex;

use crate::arch::addr::{KPhysAddr, align_up};
use crate::arch::layout::PAGE_SIZE;
use crate::boot::{KBootInfo, KMemoryMapType};

static PMM: SpinMutex<Option<BitmapAllocator>> = SpinMutex::new(None);

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

fn page_to_mb(page: usize) -> usize { (page * PAGE_SIZE) / (1024 * 1024) }

pub fn install() {
    let boot_info = KBootInfo::get();
    let memory_map = boot_info.memory.lock();

    log::debug!("installing pmm");
    let mut highest_addr: KPhysAddr = KPhysAddr::new(0);

    for entry in memory_map.iter() {
        if entry.usage == KMemoryMapType::Usable {
            let entry_end = entry.end();
            if entry_end > highest_addr {
                highest_addr = entry_end;
            }
        }
    }

    log::debug!("highest memory address: {:#x}", highest_addr);

    let total_pages = (highest_addr.as_u64() as usize).div_ceil(PAGE_SIZE);
    let bitmap_size = total_pages.div_ceil(8);

    let mut bitmap_addr: Option<u64> = None;
    for entry in memory_map.iter() {
        let aligned_base = align_up(entry.address.as_u64(), PAGE_SIZE as u64);
        if entry.usage == KMemoryMapType::Usable
            && entry.length as u64
                >= (aligned_base - entry.address.as_u64()) + bitmap_size as u64
        {
            bitmap_addr = Some(aligned_base);
            break;
        }
    }

    let bitmap_addr = bitmap_addr.expect("no space for pmm bitmap");
    let bitmap_ptr =
        (bitmap_addr + boot_info.hhdm.unwrap().as_u64()) as *mut u8;

    log::debug!(
        "Bitmap address: {:#x}, size: {} bytes",
        bitmap_addr,
        bitmap_size
    );

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

    for entry in memory_map.iter() {
        if entry.usage == KMemoryMapType::Usable {
            let start_page =
                (entry.address.as_u64() as usize).div_ceil(PAGE_SIZE);
            let end_page =
                (entry.address.as_u64() as usize + entry.length) / PAGE_SIZE;

            for page in start_page..end_page {
                allocator.clear_bit(page);
                allocator.free_pages += 1;
            }
        }
    }

    let bitmap_start_page = (bitmap_addr as usize) / PAGE_SIZE;
    let bitmap_end_page =
        ((bitmap_addr as usize) + bitmap_size).div_ceil(PAGE_SIZE);

    for page in bitmap_start_page..bitmap_end_page {
        if !allocator.test_bit(page) {
            allocator.set_bit(page);
            allocator.free_pages -= 1;
        }
    }

    allocator.usable_pages = allocator.free_pages;

    log::info!("PMM installed.");
    log::info!(
        "PMM: total pages: {}MiB, usable pages: {}MiB, free pages: {}MiB.",
        page_to_mb(total_pages),
        page_to_mb(allocator.usable_pages),
        page_to_mb(allocator.free_pages)
    );

    *PMM.lock() = Some(allocator);
}

pub fn alloc() -> Option<u64> {
    if let Some(pmm) = PMM.lock().as_mut() { pmm.alloc_page() } else { None }
}

pub fn free(addr: u64) {
    // if address is not aligned, reject it
    if !addr.is_multiple_of(PAGE_SIZE as u64) {
        log::error!("attempted to free unaligned address: {:#x}", addr);
        return;
    }

    if let Some(pmm) = PMM.lock().as_mut() {
        pmm.free_page(addr);
    }
}

pub fn is_usable_address(addr: u64) -> bool {
    if !addr.is_multiple_of(PAGE_SIZE as u64) {
        return false;
    }

    let boot_info = KBootInfo::get();

    for entry in boot_info.memory.lock().iter() {
        if entry.usage != KMemoryMapType::Usable {
            continue;
        }

        let Some(entry_end) =
            entry.address.as_u64().checked_add(entry.length as u64)
        else {
            continue;
        };

        let Some(page_end) = addr.checked_add(PAGE_SIZE as u64) else {
            return false;
        };

        if addr >= entry.address.as_u64() && page_end <= entry_end {
            return true;
        }
    }

    false
}

pub fn max_phys_address() -> Option<u64> {
    PMM.lock().as_ref().map(|pmm| (pmm.total_pages * PAGE_SIZE) as u64)
}

pub fn free_pages() -> Option<usize> {
    PMM.lock().as_ref().map(|pmm| pmm.free_pages)
}

pub fn usable_pages() -> Option<usize> {
    PMM.lock().as_ref().map(|pmm| pmm.usable_pages)
}

pub fn total_pages() -> Option<usize> {
    PMM.lock().as_ref().map(|pmm| pmm.total_pages)
}
