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

//! virtual memory areas.
//!
//! a [`Vma`] describes a range of user virtual memory and, more importantly,
//! *who owns the physical frames behind it*. the page tables only know about
//! present/absent pages; they cannot tell whether a present page came from the
//! pmm, from a shared memory object, or from a device. the vma list is the
//! missing piece, and it is what makes `munmap`, `fork` and (eventually)
//! shared-memory IPC possible.
//!
//! ownership rules:
//!
//! - [`VmaKind::Anonymous`]: the current address space owns every frame.
//! - [`VmaKind::Shared`]: the [`Shmem`] object owns the frames. each mapping
//!   additionally holds one pmm reference, taken with `map_page_shared`.
//! - [`VmaKind::File`]: the filesystem/page cache owns the frames. each mapping
//!   also takes a temporary reference while it is mapped.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::Mutex;

use crate::arch::addr::KPhysAddr;
use crate::arch::generic::paging::MapFlags;
use crate::arch::layout::PAGE_SIZE;
use crate::system::mem::pmm;

static NEXT_SHMEM_ID: AtomicU64 = AtomicU64::new(0);

/// a reference-counted anonymous shared-memory object.
///
/// pages are allocated lazily and the object owns exactly one pmm reference
/// per populated page. mappings take their own references, so the frames stay
/// alive as long as either the object or a mapping needs them.
pub struct Shmem {
    id: u64,
    len: usize,
    pages: Mutex<Vec<Option<u64>>>,
}

impl Shmem {
    /// creates a shared-memory object of `len` bytes (rounded up to pages).
    pub fn new(len: usize) -> Arc<Self> {
        let pages = len.div_ceil(PAGE_SIZE);

        Arc::new(Self {
            id: NEXT_SHMEM_ID.fetch_add(1, Ordering::Relaxed),
            len: pages * PAGE_SIZE,
            pages: Mutex::new(alloc::vec![None; pages]),
        })
    }

    /// identifier used for diagnostics and future ipc handles.
    pub fn id(&self) -> u64 { self.id }

    /// size of the object in bytes, page-rounded.
    pub fn len(&self) -> usize { self.len }

    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// number of pages in the object.
    pub fn pages(&self) -> usize { self.len / PAGE_SIZE }

    /// returns the frame backing `index`, allocating and zeroing it on first
    /// use. the object keeps one pmm reference to the frame.
    pub fn page(&self, index: usize) -> Result<KPhysAddr, &'static str> {
        let mut pages = self.pages.lock();
        let slot = pages.get_mut(index).ok_or("shmem page out of range")?;

        if let Some(phys) = *slot {
            return Ok(KPhysAddr::new(phys));
        }

        let phys = pmm::alloc_zeroed().ok_or("out of memory")?;
        *slot = Some(phys);
        Ok(KPhysAddr::new(phys))
    }

    /// returns true if the page has been populated.
    pub fn is_populated(&self, index: usize) -> bool {
        self.pages.lock().get(index).is_some_and(Option::is_some)
    }
}

impl Drop for Shmem {
    fn drop(&mut self) {
        // take the pages out while holding the lock, then free them without
        // holding it. this avoids nesting the shmem lock inside the pmm lock.
        let pages = core::mem::take(&mut *self.pages.lock());

        for phys in pages.into_iter().flatten() {
            pmm::free(phys);
        }
    }
}

#[derive(Clone)]
pub enum VmaKind {
    /// private, zero-fill memory owned by the address space.
    Anonymous,
    /// shared memory object. `fork` shares the object instead of copying it.
    Shared(Arc<Shmem>),
    /// file-backed pages owned by the filesystem/page cache.
    File { phys_base: KPhysAddr },
}

#[derive(Clone)]
pub struct Vma {
    pub start: u64,
    pub end: u64,
    pub flags: MapFlags,
    pub kind: VmaKind,
}

impl Vma {
    pub fn len(&self) -> u64 { self.end - self.start }

    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }

    pub fn is_anonymous(&self) -> bool {
        matches!(self.kind, VmaKind::Anonymous)
    }
}

/// a sorted, non-overlapping list of virtual memory areas.
///
/// adjacent anonymous regions with compatible flags are merged automatically.
/// shared regions are never merged unless they refer to the same [`Shmem`].
#[derive(Clone, Default)]
pub struct VmaMap {
    regions: Vec<Vma>,
}

impl VmaMap {
    pub const fn new() -> Self { Self { regions: Vec::new() } }

    pub fn iter(&self) -> impl Iterator<Item = &Vma> { self.regions.iter() }

    pub fn len(&self) -> usize { self.regions.len() }

    pub fn is_empty(&self) -> bool { self.regions.is_empty() }

    pub fn find(&self, addr: u64) -> Option<&Vma> {
        let idx = self.regions.partition_point(|vma| vma.start <= addr);
        idx.checked_sub(1)
            .and_then(|i| self.regions.get(i))
            .filter(|vma| addr < vma.end)
    }

    /// returns true if `[start, end)` does not overlap any region.
    pub fn is_free(&self, start: u64, end: u64) -> bool {
        !self.regions.iter().any(|vma| vma.start < end && vma.end > start)
    }

    fn check_range(start: u64, end: u64) -> Result<(), &'static str> {
        if start >= end {
            return Err("invalid vma range");
        }
        if !start.is_multiple_of(PAGE_SIZE as u64)
            || !end.is_multiple_of(PAGE_SIZE as u64)
        {
            return Err("vma range is not page-aligned");
        }
        Ok(())
    }

    /// inserts a region, rejecting overlaps.
    pub fn insert(&mut self, vma: Vma) -> Result<(), &'static str> {
        Self::check_range(vma.start, vma.end)?;

        let idx = self.regions.partition_point(|r| r.start < vma.start);

        if idx > 0 {
            let prev = &self.regions[idx - 1];
            if prev.end > vma.start {
                return Err("vma overlaps an existing region");
            }
        }

        if let Some(next) = self.regions.get(idx)
            && vma.end > next.start
        {
            return Err("vma overlaps an existing region");
        }

        self.regions.insert(idx, vma);
        Ok(())
    }

    /// inserts or extends anonymous memory.
    ///
    /// regions that share a page are always merged, and their flags are or'd
    /// together because the same physical page must satisfy both mappings (this
    /// is how the elf loader handles a segment boundary that falls inside a
    /// page). merely adjacent regions are only merged when their flags are
    /// identical: merging a read-exec segment with the following read-write
    /// segment would otherwise widen the permissions of both.
    pub fn add_anonymous(
        &mut self,
        start: u64,
        end: u64,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        Self::check_range(start, end)?;

        for vma in &self.regions {
            if vma.start < end && vma.end > start && !vma.is_anonymous() {
                return Err("region overlaps a non-anonymous mapping");
            }
        }

        let mut start = start;
        let mut end = end;
        let mut flags = flags;

        let mut i = 0;
        while i < self.regions.len() {
            let vma = &self.regions[i];
            let overlaps = vma.start < end && vma.end > start;
            let adjacent = vma.start == end || vma.end == start;

            if vma.is_anonymous()
                && (overlaps || (adjacent && vma.flags == flags))
            {
                start = start.min(vma.start);
                end = end.max(vma.end);
                flags |= vma.flags;
                self.regions.remove(i);
            } else {
                i += 1;
            }
        }

        self.insert(Vma { start, end, flags, kind: VmaKind::Anonymous })
    }

    /// removes `[start, end)` from the map, splitting regions as needed.
    ///
    /// the returned pieces must be kept alive until their pages have been
    /// unmapped: dropping a shared piece may drop the last reference to the
    /// backing [`Shmem`], which frees the object's own frame references.
    pub fn remove_range(&mut self, start: u64, end: u64) -> Vec<Vma> {
        let mut removed = Vec::new();

        if start >= end {
            return removed;
        }

        let mut kept = Vec::with_capacity(self.regions.len() + 1);

        for vma in self.regions.drain(..) {
            if vma.end <= start || vma.start >= end {
                kept.push(vma);
                continue;
            }

            if vma.start < start {
                let mut left = vma.clone();
                left.end = start;
                kept.push(left);
            }

            let mut middle = vma.clone();
            middle.start = vma.start.max(start);
            middle.end = vma.end.min(end);
            removed.push(middle);

            if vma.end > end {
                let mut right = vma.clone();
                right.start = end;
                kept.push(right);
            }
        }

        self.regions = kept;
        removed
    }

    /// replaces the flags of every region overlapping `[start, end)`.
    pub fn set_flags_range(&mut self, start: u64, end: u64, flags: MapFlags) {
        let pieces = self.remove_range(start, end);

        for mut piece in pieces {
            piece.flags = flags;
            if self.insert(piece).is_err() {
                log::error!(
                    "failed to reinsert vma {:#x}-{:#x} after mprotect",
                    start,
                    end
                );
            }
        }
    }
}
