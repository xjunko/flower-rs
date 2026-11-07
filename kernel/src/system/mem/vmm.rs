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

use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::Mutex;

use crate::arch::addr::{KPhysAddr, KVirtAddr};
use crate::arch::generic::paging::MapFlags;
use crate::arch::layout::{
    KERNEL_VALLOC_END, KERNEL_VALLOC_START, USER_MMAP_BASE, USER_MMAP_END,
    USER_SPACE_END,
};
use crate::arch::{Paging, Processor};
use crate::system::mem;
use crate::system::mem::vma::{Shmem, Vma, VmaKind, VmaMap};

type Root = <Processor as Paging>::Root;

const PAGE_SIZE: usize = <Processor as Paging>::PAGE_SIZE;
const PAGE_MASK: u64 = PAGE_SIZE as u64 - 1;

static KERNEL_SPACE: Mutex<Option<AddressSpace>> = Mutex::new(None);

pub fn install() {
    const {
        assert!(
            KERNEL_VALLOC_START >= Processor::KERNEL_HALF_START,
            "kernel valloc range must live in the kernel half"
        );
    }

    Processor::paging_install();

    let root = Processor::paging_root_active();
    KERNEL_SPACE.lock().replace(AddressSpace::wrap(root));

    log::info!("vmm installed.");
    log::info!(
        "root table physical address: {:#x}.",
        Processor::paging_root_phys(root)
    );
}

pub struct AddressSpace {
    root: Root,
    owned: bool,
    vm: VmaMap,
}

impl AddressSpace {
    /// creates a new address space. the kernel half is shared with the
    /// kernel address space by `Paging::root_new`.
    pub fn new() -> Result<Self, &'static str> {
        let root = Processor::paging_root_new()?;
        Ok(Self { root, owned: true, vm: VmaMap::new() })
    }

    /// wraps an existing root table into an AddressSpace (does not own it)
    pub fn wrap(root: Root) -> Self {
        Self { root, owned: false, vm: VmaMap::new() }
    }

    /// returns the currently active address space
    pub fn current() -> Self { Self::wrap(Processor::paging_root_active()) }

    /// returns the kernel address space
    pub fn kernel() -> Self {
        let kernel_space = KERNEL_SPACE.lock();
        let kernel_space = kernel_space.as_ref().expect("vmm not installed");
        Self::wrap(kernel_space.root)
    }

    /// returns the physical address of the root table (cr3 on x86_64)
    pub fn root_phys(&self) -> u64 { Processor::paging_root_phys(self.root) }

    /// returns the architecture-specific root table handle
    pub fn root(&self) -> Root { self.root }

    /// translates a virtual address to a physical address, if it's mapped
    pub fn translate(&self, virt: KVirtAddr) -> Option<KPhysAddr> {
        Processor::paging_translate(self.root, virt).map(|(phys, _)| phys)
    }

    /// returns true if the given virtual address is mapped in this address space
    pub fn is_mapped(&self, virt: KVirtAddr) -> bool {
        self.translate(virt).is_some()
    }

    /// translates a virtual address of the currently active address space to a physical one
    pub fn virt_to_phys(virt: KVirtAddr) -> Option<KPhysAddr> {
        let hhdm = KVirtAddr::new(Processor::paging_hhdm());

        // can safely short circuit here
        if let Some(max_phys) = mem::pmm::max_phys_address()
            && let Some(hhdm_end) = hhdm.as_u64().checked_add(max_phys)
            && virt >= hhdm
            && virt.as_u64() < hhdm_end
        {
            return Some(KPhysAddr::new(virt - hhdm));
        }

        Self::current().translate(virt)
    }

    /// returns a virtual address for the given physical address in the HHDM
    pub fn phys_to_virt(phys: KPhysAddr) -> KVirtAddr {
        KVirtAddr::new(phys.as_u64() + Processor::paging_hhdm())
    }
}

// mapping/unmapping
impl AddressSpace {
    /// maps a single page at the given virtual address to the given physical
    /// address without taking a reference to the frame.
    ///
    /// this is for mmio, firmware tables and any other physical memory the
    /// pmm does not own. pair it with [`Self::unmap_page_raw`], which does not
    /// release anything.
    pub fn map_page_raw(
        &self,
        virt: KVirtAddr,
        phys: KPhysAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let virt = KVirtAddr::new(virt.as_u64() & !PAGE_MASK);
        let phys = KPhysAddr::new(phys.as_u64() & !PAGE_MASK);

        Processor::paging_map(self.root, virt, phys, flags).inspect_err(
            |e| {
                log::error!(
                    "map failed virt={:#x} phys={:#x}: {}",
                    virt,
                    phys,
                    e
                );
            },
        )?;

        self.flush(virt);
        Ok(())
    }

    /// maps `virt` to `phys`, transferring one caller-owned pmm reference to
    /// the mapping. use [`Self::map_page_shared`] to keep the caller's
    /// reference, or [`Self::map_page_raw`] for non-pmm memory.
    pub fn map_page(
        &self,
        virt: KVirtAddr,
        phys: KPhysAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let phys = KPhysAddr::new(phys.as_u64() & !PAGE_MASK);

        if let Err(e) = self.map_page_raw(virt, phys, flags) {
            // the caller handed the reference to us, so we still own it when
            // the mapping fails.
            mem::pmm::free(phys.as_u64());
            return Err(e);
        }

        Ok(())
    }

    /// maps `virt` to `phys`, taking an additional pmm reference first. the
    /// caller keeps its own reference; the mapping owns a second one that
    /// `unmap_page` releases.
    pub fn map_page_shared(
        &self,
        virt: KVirtAddr,
        phys: KPhysAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let phys = KPhysAddr::new(phys.as_u64() & !PAGE_MASK);

        mem::pmm::retain(phys.as_u64())
            .map_err(|_| "cannot share a frame that is not pmm-managed")?;

        if let Err(e) = self.map_page_raw(virt, phys, flags) {
            // undo the reference we just took.
            mem::pmm::free(phys.as_u64());
            return Err(e);
        }

        Ok(())
    }

    /// maps a single page at `virt` to a newly allocated, zeroed physical
    /// frame. the mapping owns the frame's only reference, which `unmap_page`
    /// releases.
    pub fn map_page_alloc(
        &self,
        virt: KVirtAddr,
        flags: MapFlags,
    ) -> Result<KPhysAddr, &'static str> {
        let phys = KPhysAddr::new(mem::pmm::alloc_zeroed().ok_or("oom")?);

        if let Err(e) = self.map_page_raw(virt, phys, flags) {
            log::error!(
                "failed to map page virt={:#x}, phys={:#x}, flags={:?}: {}",
                virt,
                phys,
                flags,
                e
            );
            mem::pmm::free(phys.as_u64());
            return Err(e);
        }

        Ok(phys)
    }

    /// maps a range of virtual addresses to newly allocated physical pages with the specified flags
    pub fn map_range_alloc(
        &self,
        virt: KVirtAddr,
        size: usize,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let (start, end) = Self::page_span(virt, size)?;

        let mut page = start;
        loop {
            if let Err(e) = self.map_page_alloc(page, flags) {
                log::error!(
                    "map_range_alloc failed at {:#x} (start={:#x}, size={:#x}): {}, rolling back",
                    page,
                    virt,
                    size,
                    e
                );

                // unmap everything we mapped before the failing page. each
                // mapping owns its frame, so unmapping releases it.
                let mut done = start;
                while done < page {
                    let _ = self.unmap_page(done);
                    done += PAGE_SIZE as u64;
                }

                return Err("failed to map range in address space");
            }

            if page == end {
                break;
            }
            page += PAGE_SIZE as u64;
        }

        Ok(())
    }

    /// unmaps the managed page at the given virtual address, releases the
    /// reference the mapping owns, and returns the physical address that was
    /// mapped there.
    pub fn unmap_page(
        &self,
        virt: KVirtAddr,
    ) -> Result<KPhysAddr, &'static str> {
        let phys = self.unmap_page_raw(virt)?;
        mem::pmm::free(phys.as_u64());
        Ok(phys)
    }

    /// unmaps the page at the given virtual address without releasing the
    /// frame. pair it with [`Self::map_page_raw`].
    pub fn unmap_page_raw(
        &self,
        virt: KVirtAddr,
    ) -> Result<KPhysAddr, &'static str> {
        let virt = KVirtAddr::new(virt.as_u64() & !PAGE_MASK);
        let phys = Processor::paging_unmap(self.root, virt)?;
        self.flush(virt);
        Ok(phys)
    }

    /// unmaps a range of managed virtual addresses and releases every frame
    /// the mappings own.
    ///
    /// every page is attempted even if one of them fails, so a single bad
    /// mapping cannot leave the rest of the range behind.
    pub fn unmap_range(
        &self,
        virt: KVirtAddr,
        size: usize,
    ) -> Result<(), &'static str> {
        let (start, end) = Self::page_span(virt, size)?;

        let mut first_error = None;
        let mut page = start;
        loop {
            if let Err(e) = self.unmap_page(page) {
                first_error.get_or_insert(e);
            }

            if page == end {
                break;
            }
            page += PAGE_SIZE as u64;
        }

        match first_error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// returns the page flags for the given virtual address, or an error if it's not mapped
    pub fn page_flags(
        &self,
        virt: KVirtAddr,
    ) -> Result<MapFlags, &'static str> {
        Processor::paging_translate(
            self.root,
            KVirtAddr::new(virt.as_u64() & !PAGE_MASK),
        )
        .map(|(_, flags)| flags)
        .ok_or("page not mapped")
    }

    /// updates the page flags for the given virtual address, returns an error if it's not mapped or if the update fails
    pub fn update_page_flags(
        &self,
        virt: KVirtAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let virt = KVirtAddr::new(virt.as_u64() & !PAGE_MASK);
        Processor::paging_update(self.root, virt, flags)?;
        self.flush(virt);
        Ok(())
    }
}

// vmem alloc
//
// this is a reclaiming, page-granular allocator over the kernel valloc window.
// it is deliberately simple (first-fit over a sorted free list) because kernel
// virtual allocations are rare: stacks, device windows and similar one-off
// reservations. freeing coalesces adjacent ranges so repeated
// create/destroy cycles do not fragment the window.

#[derive(Clone, Copy)]
struct FreeRange {
    start: u64,
    end: u64,
}

struct VirtSpace {
    free: Vec<FreeRange>,
    initialized: bool,
}

impl VirtSpace {
    fn ensure_initialized(&mut self) {
        if !self.initialized {
            self.free.push(FreeRange {
                start: KERNEL_VALLOC_START,
                end: KERNEL_VALLOC_END,
            });
            self.initialized = true;
        }
    }

    fn alloc(&mut self, size: u64) -> Option<u64> {
        self.ensure_initialized();

        for i in 0..self.free.len() {
            let range = self.free[i];
            let available = range.end - range.start;
            if available < size {
                continue;
            }

            let start = range.start;

            if available == size {
                self.free.remove(i);
            } else {
                self.free[i].start = start + size;
            }

            return Some(start);
        }

        None
    }

    fn release(&mut self, start: u64, end: u64) -> Result<(), &'static str> {
        self.ensure_initialized();

        if start >= end {
            return Err("invalid valloc range");
        }

        if start < KERNEL_VALLOC_START || end > KERNEL_VALLOC_END {
            return Err("valloc range is outside the kernel window");
        }

        if !start.is_multiple_of(PAGE_SIZE as u64)
            || !end.is_multiple_of(PAGE_SIZE as u64)
        {
            return Err("valloc range is not page-aligned");
        }

        let idx = self.free.partition_point(|range| range.start < start);

        if idx > 0 && self.free[idx - 1].end > start {
            return Err("valloc range is already free");
        }

        if idx < self.free.len() && self.free[idx].start < end {
            return Err("valloc range is already free");
        }

        self.free.insert(idx, FreeRange { start, end });

        // merge with the following range
        if idx + 1 < self.free.len()
            && self.free[idx].end == self.free[idx + 1].start
        {
            self.free[idx].end = self.free[idx + 1].end;
            self.free.remove(idx + 1);
        }

        // merge with the preceding range
        if idx > 0 && self.free[idx - 1].end == self.free[idx].start {
            self.free[idx - 1].end = self.free[idx].end;
            self.free.remove(idx);
        }

        Ok(())
    }
}

static KERNEL_VALLOC: Mutex<VirtSpace> =
    Mutex::new(VirtSpace { free: Vec::new(), initialized: false });

impl AddressSpace {
    /// reserves a range of virtual addresses in the kernel valloc window.
    /// the range is not mapped; pair it with [`Self::release_virt`] when it is
    /// no longer needed.
    pub fn reserve_virt(size: usize) -> Result<KVirtAddr, &'static str> {
        if size == 0 {
            return Err("invalid size");
        }

        let pages = size.div_ceil(PAGE_SIZE) as u64;
        let bytes = pages
            .checked_mul(PAGE_SIZE as u64)
            .ok_or("valloc size overflow")?;

        let start = KERNEL_VALLOC
            .lock()
            .alloc(bytes)
            .ok_or("valloc space exhausted")?;

        Ok(KVirtAddr::new(start))
    }

    /// returns a range previously obtained from [`Self::reserve_virt`].
    /// adjacent reservations are coalesced.
    pub fn release_virt(
        virt: KVirtAddr,
        size: usize,
    ) -> Result<(), &'static str> {
        let pages = size.div_ceil(PAGE_SIZE) as u64;
        let bytes = pages
            .checked_mul(PAGE_SIZE as u64)
            .ok_or("valloc size overflow")?;
        let start = virt.as_u64();
        let end = start.checked_add(bytes).ok_or("valloc size overflow")?;

        KERNEL_VALLOC.lock().release(start, end)
    }

    /// allocates `size` bytes of virtual memory and maps it to fresh frames.
    pub fn alloc_virt(
        &self,
        size: usize,
        flags: MapFlags,
    ) -> Result<KVirtAddr, &'static str> {
        let virt = Self::reserve_virt(size)?;

        if let Err(e) = self.map_range_alloc(virt, size, flags) {
            // give the reservation back so a failed allocation does not burn
            // address space forever.
            let _ = Self::release_virt(virt, size);
            log::error!(
                "failed to map range virt={:#x}, size={:#x}, flags={:?}: {}",
                virt,
                size,
                flags,
                e
            );
            return Err(e);
        }

        Ok(virt)
    }

    /// unmaps a range allocated with [`Self::alloc_virt`] and returns the
    /// virtual addresses to the valloc window.
    pub fn free_virt(
        &self,
        virt: KVirtAddr,
        size: usize,
    ) -> Result<(), &'static str> {
        self.unmap_range(virt, size)?;
        Self::release_virt(virt, size)
    }
}

// user vma operations
impl AddressSpace {
    /// rejects null addresses and anything outside the canonical user half.
    fn check_user_range(start: u64, end: u64) -> Result<(), &'static str> {
        if start >= end {
            return Err("invalid user range");
        }

        if start < PAGE_SIZE as u64 {
            return Err("user range touches the null page");
        }

        if end > USER_SPACE_END {
            return Err("user range leaves the user half");
        }

        Ok(())
    }

    /// the vma list belonging to this address space.
    ///
    /// only owned (process) address spaces carry useful vmas. the temporary
    /// wrappers returned by [`Self::current`] and [`Self::kernel`] always have
    /// an empty list.
    pub fn vma(&self) -> &VmaMap { &self.vm }

    pub fn vma_mut(&mut self) -> &mut VmaMap { &mut self.vm }

    pub fn vma_at(&self, addr: KVirtAddr) -> Option<&Vma> {
        self.vm.find(addr.as_u64())
    }

    /// registers an anonymous range without mapping it. used by the elf loader
    /// and the initial heap/stack setup.
    pub fn add_anonymous_vma(
        &mut self,
        start: u64,
        end: u64,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        self.vm.add_anonymous(start, end, flags)
    }

    /// finds a free `len`-byte range inside the user mmap window.
    pub fn find_free_user_range(&self, len: usize) -> Option<u64> {
        let len = len as u64;
        let mut cursor = USER_MMAP_BASE;

        for vma in self.vm.iter() {
            if vma.end <= cursor {
                continue;
            }

            if vma.start >= USER_MMAP_END {
                break;
            }

            if vma.start > cursor && vma.start - cursor >= len {
                return Some(cursor);
            }

            cursor = cursor.max(vma.end);

            if cursor >= USER_MMAP_END {
                return None;
            }
        }

        (USER_MMAP_END - cursor >= len).then_some(cursor)
    }

    /// returns true if `[start, start + len)` is entirely free: no vma covers
    /// it and no page table entry is present.
    pub fn user_range_free(&self, start: u64, len: usize) -> bool {
        let Some(end) = start.checked_add(len as u64) else {
            return false;
        };

        if start < PAGE_SIZE as u64 || end > USER_SPACE_END {
            return false;
        }

        if !self.vm.is_free(start, end) {
            return false;
        }

        let mut page = start;
        while page < end {
            if self.is_mapped(KVirtAddr::new(page)) {
                return false;
            }
            page += PAGE_SIZE as u64;
        }

        true
    }

    /// maps a private, zero-filled anonymous range and records its vma.
    pub fn map_user_anonymous(
        &mut self,
        start: u64,
        len: usize,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let end = start.checked_add(len as u64).ok_or("user range overflow")?;
        Self::check_user_range(start, end)?;

        self.vm.add_anonymous(start, end, flags)?;

        if let Err(e) = self.map_range_alloc(KVirtAddr::new(start), len, flags)
        {
            // undo the metadata before returning.
            let _ = self.unmap_user_range(start, end);
            return Err(e);
        }

        Ok(())
    }

    /// creates a shared-memory object and maps it into this address space.
    /// the object is kept alive by the vma and by every mapping of it.
    pub fn map_user_shared(
        &mut self,
        start: u64,
        len: usize,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        self.map_user_shmem(start, len, flags, Shmem::new(len))
    }

    /// maps an existing shared-memory object into this address space.
    ///
    /// this is the building block for future `shm_open`/socket IPC: the
    /// creator keeps an `Arc<Shmem>` handle, and other processes map the same
    /// object here instead of allocating a private copy.
    pub fn map_user_shmem(
        &mut self,
        start: u64,
        len: usize,
        flags: MapFlags,
        shmem: Arc<Shmem>,
    ) -> Result<(), &'static str> {
        let end = start.checked_add(len as u64).ok_or("user range overflow")?;
        Self::check_user_range(start, end)?;

        if len > shmem.len() {
            return Err("shared mapping is larger than the shmem object");
        }

        self.vm.insert(Vma {
            start,
            end,
            flags,
            kind: VmaKind::Shared(shmem.clone()),
        })?;

        let pages = len.div_ceil(PAGE_SIZE);

        for index in 0..pages {
            let virt = KVirtAddr::new(start + (index * PAGE_SIZE) as u64);

            let mapped = shmem
                .page(index)
                .and_then(|phys| self.map_page_shared(virt, phys, flags));

            if let Err(e) = mapped {
                let _ = self.unmap_user_range(start, end);
                return Err(e);
            }
        }

        Ok(())
    }

    /// maps pre-existing physical pages owned by a file/page cache.
    /// each mapping takes its own pmm reference; the filesystem keeps its
    /// reference until the cache entry is evicted.
    pub fn map_user_file(
        &mut self,
        start: u64,
        len: usize,
        flags: MapFlags,
        phys_base: KPhysAddr,
    ) -> Result<(), &'static str> {
        let end = start.checked_add(len as u64).ok_or("user range overflow")?;
        Self::check_user_range(start, end)?;

        self.vm.insert(Vma {
            start,
            end,
            flags,
            kind: VmaKind::File { phys_base },
        })?;

        let pages = len.div_ceil(PAGE_SIZE);

        for index in 0..pages {
            let virt = KVirtAddr::new(start + (index * PAGE_SIZE) as u64);
            let phys =
                KPhysAddr::new(phys_base.as_u64() + (index * PAGE_SIZE) as u64);

            if let Err(e) = self.map_page_shared(virt, phys, flags) {
                let _ = self.unmap_user_range(start, end);
                return Err(e);
            }
        }

        Ok(())
    }

    /// removes `[start, end)` from the vma list and unmaps every present page.
    ///
    /// the removed vmas are dropped only after the pages have been unmapped so
    /// a shared object's frames are not released before its mappings are.
    pub fn unmap_user_range(
        &mut self,
        start: u64,
        end: u64,
    ) -> Result<(), &'static str> {
        if start >= end {
            return Ok(());
        }

        if !start.is_multiple_of(PAGE_SIZE as u64)
            || !end.is_multiple_of(PAGE_SIZE as u64)
        {
            return Err("user range is not page-aligned");
        }

        Self::check_user_range(start, end)?;

        let removed = self.vm.remove_range(start, end);

        let mut page = start;
        while page < end {
            let virt = KVirtAddr::new(page);

            if self.is_mapped(virt)
                && let Err(e) = self.unmap_page(virt)
            {
                log::error!("failed to unmap user page {:#x}: {}", page, e);
            }

            page += PAGE_SIZE as u64;
        }

        drop(removed);
        Ok(())
    }

    /// maps any missing pages in `[start, end)` with fresh zeroed frames.
    pub fn ensure_user_pages(
        &self,
        start: u64,
        end: u64,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        Self::check_user_range(start, end)?;

        let mut page = start;

        while page < end {
            let virt = KVirtAddr::new(page);
            if !self.is_mapped(virt) {
                self.map_page_alloc(virt, flags)?;
            }
            page += PAGE_SIZE as u64;
        }

        Ok(())
    }

    /// changes the protection of `[start, end)` in both the page tables and
    /// the vma list.
    pub fn protect_user_range(
        &mut self,
        start: u64,
        end: u64,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        Self::check_user_range(start, end)?;

        let mut page = start;

        while page < end {
            let virt = KVirtAddr::new(page);
            if self.is_mapped(virt) {
                self.update_page_flags(virt, flags)?;
            }
            page += PAGE_SIZE as u64;
        }

        self.vm.set_flags_range(start, end, flags);
        Ok(())
    }

    /// resolves a user page fault from the vma list.
    ///
    /// returns `Ok(true)` when a page was mapped, `Ok(false)` when the fault
    /// is not backed by demand-paged memory.
    pub fn demand_page(&self, page: KVirtAddr) -> Result<bool, &'static str> {
        if page.as_u64() < PAGE_SIZE as u64 || page.as_u64() >= USER_SPACE_END {
            return Ok(false);
        }

        let Some(vma) = self.vm.find(page.as_u64()) else {
            return Ok(false);
        };

        // PROT_NONE (and any region without user access) must not be silently
        // populated on a user fault.
        if !vma.flags.contains(MapFlags::USER) {
            return Ok(false);
        }

        if self.is_mapped(page) {
            return Ok(false);
        }

        match &vma.kind {
            VmaKind::Anonymous => {
                self.map_page_alloc(page, vma.flags)?;
                Ok(true)
            },
            VmaKind::Shared(shmem) => {
                let index =
                    ((page.as_u64() - vma.start) / PAGE_SIZE as u64) as usize;
                let phys = shmem.page(index)?;
                self.map_page_shared(page, phys, vma.flags)?;
                Ok(true)
            },
            VmaKind::File { .. } => Ok(false),
        }
    }

    /// for every shared vma, replaces the private pages created by
    /// `paging_clone_user` with mappings of the parent's shared frames.
    fn rebind_shared(&self, parent: &Self) -> Result<(), &'static str> {
        for vma in self.vm.iter() {
            let VmaKind::Shared(shmem) = &vma.kind else {
                continue;
            };

            // a vma may map only a prefix of the object, so walk the vma's
            // pages rather than the object's. walking the object would rebind
            // (and potentially clobber) addresses outside this vma.
            let pages = ((vma.end - vma.start) / PAGE_SIZE as u64) as usize;

            for index in 0..pages {
                let virt =
                    KVirtAddr::new(vma.start + (index * PAGE_SIZE) as u64);

                // only pages the parent actually had mapped should be
                // rebound; the child's copy is private and can be released.
                if !parent.is_mapped(virt) {
                    continue;
                }

                if self.is_mapped(virt) {
                    self.unmap_page(virt)?;
                }

                let phys = shmem.page(index)?;
                self.map_page_shared(virt, phys, vma.flags)?;
            }
        }

        Ok(())
    }
}

// utils
impl AddressSpace {
    /// returns the first and last page (inclusive) touched by `[virt, virt + size)`
    fn page_span(
        virt: KVirtAddr,
        size: usize,
    ) -> Result<(KVirtAddr, KVirtAddr), &'static str> {
        if size == 0 {
            return Err("invalid size");
        }

        let last = virt
            .as_u64()
            .checked_add(size as u64 - 1)
            .ok_or("address range overflow")?;
        Ok((
            KVirtAddr::new(virt.as_u64() & !PAGE_MASK),
            KVirtAddr::new(last & !PAGE_MASK),
        ))
    }

    /// flushes the tlb entry for `virt` if this address space can be cached on this cpu.
    /// the kernel half is shared between every address space, so it is always flushed.
    fn flush(&self, virt: KVirtAddr) {
        if virt >= KVirtAddr::new(Processor::KERNEL_HALF_START)
            || self.root == Processor::paging_root_active()
        {
            Processor::paging_flush(virt);
        }
    }

    /// translates a (possibly unaligned) virtual address to its physical address
    fn phys_of(&self, virt: KVirtAddr) -> Result<KPhysAddr, &'static str> {
        let (phys, _) = Processor::paging_translate(
            self.root,
            KVirtAddr::new(virt.as_u64() & !PAGE_MASK),
        )
        .ok_or("page not mapped")?;
        Ok(phys + (virt.as_u64() & PAGE_MASK))
    }
}

// copying/zero
impl AddressSpace {
    /// zeros the given range of virtual addresses, returns an error if any page in the range is not mapped
    pub fn zero(
        &self,
        virt: KVirtAddr,
        len: usize,
    ) -> Result<(), &'static str> {
        let mut offset = 0;

        while offset < len {
            let current = virt + offset as u64;
            let page_offset = (current.as_u64() & PAGE_MASK) as usize;
            let bytes_in_page =
                core::cmp::min(PAGE_SIZE - page_offset, len - offset);

            let phys = self.phys_of(current)?;

            unsafe {
                core::ptr::write_bytes(
                    Self::phys_to_virt(phys).as_mut_ptr::<u8>(),
                    0,
                    bytes_in_page,
                );
            }

            offset += bytes_in_page;
        }

        Ok(())
    }

    /// writes the given data to the given virtual address, returns an error if any page in the range is not mapped
    pub fn write(
        &self,
        virt: KVirtAddr,
        data: &[u8],
    ) -> Result<(), &'static str> {
        let mut offset = 0;

        while offset < data.len() {
            let current = virt + offset as u64;
            let page_offset = (current.as_u64() & PAGE_MASK) as usize;
            let bytes_in_page =
                core::cmp::min(PAGE_SIZE - page_offset, data.len() - offset);

            let phys = self.phys_of(current)?;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(offset),
                    Self::phys_to_virt(phys).as_mut_ptr::<u8>(),
                    bytes_in_page,
                );
            }

            offset += bytes_in_page;
        }

        Ok(())
    }
}

// userspace stuff
impl AddressSpace {
    /// creates a new address space with a copy of the user half of this one.
    ///
    /// private pages are deep-copied by the paging layer. shared vmas are then
    /// rebound to the parent's shared frames, so `fork` gets real `MAP_SHARED`
    /// semantics instead of a private copy.
    pub fn clone_user(&self) -> Result<Self, &'static str> {
        let root = Processor::paging_clone_user(self.root)?;
        let child = Self { root, owned: true, vm: self.vm.clone() };

        if let Err(e) = child.rebind_shared(self) {
            // dropping the child frees its page tables (releasing the
            // mappings' frame references) and then its shared vmas.
            return Err(e);
        }

        Ok(child)
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        if !self.owned {
            return;
        }

        if self.root == Processor::paging_root_active() {
            log::error!(
                "refusing to free active address space root at {:#x}",
                Processor::paging_root_phys(self.root)
            );
            return;
        }

        if KERNEL_SPACE
            .lock()
            .as_ref()
            .is_some_and(|kernel| kernel.root == self.root)
        {
            log::error!(
                "refusing to free kernel address space root at {:#x}",
                Processor::paging_root_phys(self.root)
            );
            return;
        }

        let phys = Processor::paging_root_phys(self.root);

        // frees the user half tables/frames and the root itself
        Processor::paging_root_free(self.root);

        log::trace!("dropped address space and freed root at {:#x}", phys);
    }
}
