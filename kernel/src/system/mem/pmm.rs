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
use crate::arch::generic::interrupts;
use crate::arch::layout::PAGE_SIZE;
use crate::boot::{KBootInfo, KMemoryMapType};

const PFN_NONE: u32 = u32::MAX;
const PFN_MAX_COUNT: u16 = u16::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum PageState {
    Reserved = 0,
    Free = 1,
    Allocated = 2,
    Bad = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PfnEntry {
    next: u32,
    refcount: u16,
    state: PageState,
    _pad: u8,
}

const _: () = assert!(core::mem::size_of::<PfnEntry>() == 8);

struct PfnDb {
    entries: *mut PfnEntry,
    total_pages: usize,
    usable_pages: usize,
    free_pages: usize,
    free_head: u32,
}

unsafe impl Send for PfnDb {}
unsafe impl Sync for PfnDb {}

impl PfnDb {
    #[inline]
    fn entry(&self, pfn: u32) -> &PfnEntry {
        debug_assert!((pfn as usize) < self.total_pages);
        unsafe { &*self.entries.add(pfn as usize) }
    }

    #[inline]
    fn entry_mut(&mut self, pfn: u32) -> &mut PfnEntry {
        debug_assert!((pfn as usize) < self.total_pages);
        unsafe { &mut *self.entries.add(pfn as usize) }
    }

    #[inline]
    fn pfn_of(&self, addr: u64) -> Option<u32> {
        if !addr.is_multiple_of(PAGE_SIZE as u64) {
            return None;
        }

        let pfn = addr / PAGE_SIZE as u64;
        if pfn >= self.total_pages as u64 || pfn > u32::MAX as u64 {
            return None;
        }

        Some(pfn as u32)
    }

    fn mark_free(&mut self, pfn: u32) {
        let free_head = self.free_head;
        let entry = self.entry_mut(pfn);
        if entry.state != PageState::Reserved {
            return;
        }

        entry.state = PageState::Free;
        entry.refcount = 0;
        entry.next = free_head;
        self.free_head = pfn;
        self.free_pages += 1;
    }

    fn pop_free(&mut self) -> Option<u32> {
        let pfn = self.free_head;
        if pfn == PFN_NONE {
            return None;
        }

        let entry = self.entry_mut(pfn);
        debug_assert_eq!(entry.state, PageState::Free);

        let next = entry.next;
        entry.next = PFN_NONE;
        entry.state = PageState::Allocated;
        entry.refcount = 1;

        self.free_head = next;
        self.free_pages -= 1;

        Some(pfn)
    }

    fn release(&mut self, pfn: u32) -> Result<(), &'static str> {
        let free_head = self.free_head;
        let entry = self.entry_mut(pfn);

        match entry.state {
            PageState::Allocated => {},
            PageState::Free => return Err("frame is already free"),
            PageState::Reserved => return Err("frame is reserved"),
            PageState::Bad => return Err("frame is unusable"),
        }

        if entry.refcount > 1 {
            entry.refcount -= 1;
            return Ok(());
        }

        if entry.refcount == 0 {
            return Err("allocated frame has zero references");
        }

        entry.refcount = 0;
        entry.state = PageState::Free;
        entry.next = free_head;

        self.free_head = pfn;
        self.free_pages += 1;

        Ok(())
    }

    fn retain(&mut self, pfn: u32) -> Result<(), &'static str> {
        let entry = self.entry_mut(pfn);

        if entry.state != PageState::Allocated {
            return Err("cannot retain a frame that is not allocated");
        }

        if entry.refcount == PFN_MAX_COUNT {
            return Err("frame refcount overflow");
        }

        entry.refcount += 1;
        Ok(())
    }
}

static PMM: SpinMutex<Option<PfnDb>> = SpinMutex::new(None);

pub fn install() {
    let boot_info = KBootInfo::get();
    let memory_map = boot_info.memory.lock();

    log::debug!("pmm: installing pmm");

    let highest_usable = memory_map
        .iter()
        .filter(|entry| entry.usage == KMemoryMapType::Usable)
        .map(|entry| entry.address.as_u64().saturating_add(entry.length as u64))
        .max()
        .expect("no usable memory");

    let total_pages = (highest_usable as usize).div_ceil(PAGE_SIZE);

    assert!(total_pages < PFN_NONE as usize, "too many pages");

    let entry_size = core::mem::size_of::<PfnEntry>();
    let db_size =
        total_pages.checked_mul(entry_size).expect("pfndb size overflow");

    // find the addr that fits our db_size
    let mut db_addr: Option<u64> = None;
    for entry in memory_map.iter() {
        if entry.usage != KMemoryMapType::Usable {
            continue;
        }

        let base = entry.address.as_u64();
        let end = base.saturating_add(entry.length as u64);
        let aligned = align_up(base, PAGE_SIZE as u64);

        let Some(db_end) = aligned.checked_add(db_size as u64) else {
            continue;
        };

        if db_end <= end {
            db_addr = Some(aligned);
            break;
        }
    }

    let db_addr = db_addr.expect("no space for pmm pfndb");
    let hhdm = boot_info.hhdm.expect("no hhdm").as_u64();
    let db_ptr = (db_addr + hhdm) as *mut PfnEntry;

    log::debug!(
        "pmm: pfndb at {:#x}, {} entries ({} bytes)",
        db_addr,
        total_pages,
        db_size
    );

    unsafe {
        core::ptr::write_bytes(db_ptr as *mut u8, 0, db_size);
    }

    let db_start_pfn = (db_addr / PAGE_SIZE as u64) as usize;
    let db_end_pfn =
        ((db_addr + db_size as u64).div_ceil(PAGE_SIZE as u64)) as usize;

    let mut allocator = PfnDb {
        entries: db_ptr,
        total_pages,
        usable_pages: 0,
        free_pages: 0,
        free_head: PFN_NONE,
    };

    for entry in memory_map.iter() {
        if entry.usage != KMemoryMapType::Usable {
            continue;
        }

        let base = entry.address.as_u64();
        let start_page = (base as usize).div_ceil(PAGE_SIZE);
        let end_page =
            ((base as usize).saturating_add(entry.length)) / PAGE_SIZE;
        let end_page = end_page.min(total_pages);

        for page in start_page..end_page {
            // pfndb lives here!!
            if page >= db_start_pfn && page < db_end_pfn {
                continue;
            }

            allocator.mark_free(page as u32);
        }
    }

    allocator.usable_pages = allocator.free_pages;

    {
        fn page_to_mb(page: usize) -> usize {
            (page * PAGE_SIZE) / (1024 * 1024)
        }

        log::info!("pmm: installed.");
        log::info!(
            "pmm: total: {}MiB, usable: {}MiB, free: {}MiB.",
            page_to_mb(total_pages),
            page_to_mb(allocator.usable_pages),
            page_to_mb(allocator.free_pages)
        );
    }

    *PMM.lock() = Some(allocator);
}

pub fn alloc() -> Option<u64> {
    interrupts::without(|| {
        let mut pmm = PMM.lock();
        let pfn = pmm.as_mut()?.pop_free()?;
        Some(pfn as u64 * PAGE_SIZE as u64)
    })
}

/// allocates a single, page-aligned and zeroed physical frame.
pub fn alloc_zeroed() -> Option<u64> {
    let addr = alloc()?;

    // the frame is no longer reachable by anyone else, so clearing it outside
    // the pmm lock is safe.
    unsafe {
        core::ptr::write_bytes(
            KPhysAddr::new(addr).as_hhdm::<u8>(),
            0,
            PAGE_SIZE,
        );
    }

    Some(addr)
}

pub fn free(addr: u64) {
    if !addr.is_multiple_of(PAGE_SIZE as u64) {
        log::error!("attempted to free unaligned address: {:#x}", addr);
        return;
    }

    interrupts::without(|| {
        let mut pmm = PMM.lock();
        let Some(db) = pmm.as_mut() else {
            log::error!("pmm: free({:#x}) before install", addr);
            return;
        };

        let Some(pfn) = db.pfn_of(addr) else {
            log::error!("pmm: free({:#x}) out of range", addr);
            return;
        };

        if let Err(reason) = db.release(pfn) {
            log::error!("pmm: free({:#x}) rejected: {}", addr, reason);
        }
    });
}

pub fn retain(addr: u64) -> Result<(), &'static str> {
    if !addr.is_multiple_of(PAGE_SIZE as u64) {
        return Err("attempted to retain an unaligned address");
    }

    interrupts::without(|| {
        let mut pmm = PMM.lock();
        let db = pmm.as_mut().ok_or("pmm not installed")?;
        let pfn = db.pfn_of(addr).ok_or("address out of range")?;
        db.retain(pfn)
    })
}

pub fn is_usable_address(addr: u64) -> bool {
    interrupts::without(|| {
        let pmm = PMM.lock();
        let Some(db) = pmm.as_ref() else {
            return false;
        };
        let Some(pfn) = db.pfn_of(addr) else {
            return false;
        };

        matches!(db.entry(pfn).state, PageState::Free | PageState::Allocated)
    })
}

pub fn max_phys_address() -> Option<u64> {
    interrupts::without(|| {
        PMM.lock().as_ref().map(|pmm| (pmm.total_pages * PAGE_SIZE) as u64)
    })
}

pub fn free_pages() -> Option<usize> {
    interrupts::without(|| PMM.lock().as_ref().map(|pmm| pmm.free_pages))
}

pub fn usable_pages() -> Option<usize> {
    interrupts::without(|| PMM.lock().as_ref().map(|pmm| pmm.usable_pages))
}

pub fn total_pages() -> Option<usize> {
    interrupts::without(|| PMM.lock().as_ref().map(|pmm| pmm.total_pages))
}
