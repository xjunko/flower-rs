use spin::Mutex;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::TranslateResult;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
    PhysFrame, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::{boot, system};

static HHDM: Mutex<Option<u64>> = Mutex::new(None);
static PML4: Mutex<Option<PhysAddr>> = Mutex::new(None);

pub struct PMMFrameAllocator;
unsafe impl FrameAllocator<Size4KiB> for PMMFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let addr = system::mem::pmm::alloc()?;
        Some(PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

pub fn install() {
    *HHDM.lock() =
        Some(boot::limine::HHDM_REQUEST.get_response().unwrap().offset());

    let (pml4_frame, _) = Cr3::read();
    *PML4.lock() = Some(pml4_frame.start_address());

    log::info!("VMM installed.");
    log::info!("PML4 physical address: {:#x}.", pml4_frame.start_address());
}

fn hhdm() -> u64 { HHDM.lock().expect("no hhdm") }

/// translates a physical address in the HHDM to a virtual address
pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys.as_u64() + hhdm())
}

/// translates a virtual address to a physical address, if it's mapped and not in the HHDM
pub fn virt_to_phys(virt: VirtAddr) -> Option<PhysAddr> {
    let hhdm = hhdm();

    if let Some(max_phys) = system::mem::pmm::max_phys_address()
        && let Some(hhdm_end) = hhdm.checked_add(max_phys)
        && virt.as_u64() >= hhdm
        && virt.as_u64() < hhdm_end
    {
        return Some(PhysAddr::new(virt.as_u64() - hhdm));
    }

    log::debug!(
        "virt_to_phys: address {:#x} is below hhdm {:#x}, translation needed",
        virt.as_u64(),
        hhdm
    );

    unsafe {
        let mapper = page_get_current_table();
        mapper.translate_addr(virt)
    }
}

/// gets the currently active page table and returns an OffsetPageTable for it
unsafe fn page_get_current_table() -> OffsetPageTable<'static> {
    let (pml4_frame, _) = Cr3::read();
    let pml4_virt = phys_to_virt(pml4_frame.start_address());
    let pml4: &'static mut PageTable = unsafe { &mut *pml4_virt.as_mut_ptr() };
    unsafe { OffsetPageTable::new(pml4, VirtAddr::new(hhdm())) }
}

/// gets the page table at the given physical address and returns an OffsetPageTable for it
unsafe fn page_get_table_at(pml4_phys: PhysAddr) -> OffsetPageTable<'static> {
    let pml4_virt = phys_to_virt(pml4_phys);
    let pml4: &'static mut PageTable = unsafe { &mut *pml4_virt.as_mut_ptr() };
    unsafe { OffsetPageTable::new(pml4, VirtAddr::new(hhdm())) }
}

/// maps a single page at the given virtual address to the given physical address with the specified flags
pub fn page_map(
    virt: VirtAddr,
    phys: PhysAddr,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    let page: Page<Size4KiB> = Page::containing_address(virt);
    let frame = PhysFrame::containing_address(phys);

    unsafe {
        let mut mapper = page_get_current_table();
        let mut allocator = PMMFrameAllocator;

        mapper
            .map_to(page, frame, flags, &mut allocator)
            .map_err(|_| "failed to map page")?
            .flush();
    }

    Ok(())
}

/// maps a single page at the given virtual address to a newly allocated physical page with the specified flags
pub fn page_map_alloc(
    virt: VirtAddr,
    flags: PageTableFlags,
) -> Result<PhysAddr, &'static str> {
    let phys_addr = system::mem::pmm::alloc().ok_or("oom")?;
    let phys = PhysAddr::new(phys_addr);

    unsafe {
        let virt_ptr = phys_to_virt(phys).as_mut_ptr::<u8>();
        core::ptr::write_bytes(virt_ptr, 0, 4096);
    }

    if let Err(e) = page_map(virt, phys, flags) {
        log::error!("Failed to map page: {}", e);
        system::mem::pmm::free(phys.as_u64());
        return Err(e);
    }

    Ok(phys)
}

/// unmaps the page at the given virtual address and returns the physical address that was mapped there
pub fn page_unmap(virt: VirtAddr) -> Result<PhysAddr, &'static str> {
    log::debug!("unmapping page at virt {:#x}", virt.as_u64(),);
    let page: Page<Size4KiB> = Page::containing_address(virt);

    unsafe {
        let mut mapper = page_get_current_table();
        let (frame, flush) =
            mapper.unmap(page).map_err(|_| "failed to unmap page")?;
        flush.flush();
        Ok(frame.start_address())
    }
}

/// returns true if the given virtual address is mapped to a physical address, false otherwise
pub fn page_is_mapped(virt: VirtAddr) -> bool {
    unsafe {
        let mapper = page_get_current_table();
        mapper.translate_addr(virt).is_some()
    }
}

/// returns the page table flags for the given virtual address, or an error if it's not mapped
pub fn page_flags(virt: VirtAddr) -> Result<PageTableFlags, &'static str> {
    let page: Page<Size4KiB> = Page::containing_address(virt);

    unsafe {
        let mapper = page_get_current_table();
        match mapper.translate(page.start_address()) {
            TranslateResult::Mapped { flags, .. } => Ok(flags),
            _ => Err("page not mapped"),
        }
    }
}

/// updates the page table flags for the given virtual address, returns an error if it's not mapped or if the update fails
pub fn page_update_flags(
    virt: VirtAddr,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    let page: Page<Size4KiB> = Page::containing_address(virt);

    unsafe {
        let mut mapper = page_get_current_table();
        mapper
            .update_flags(page, flags)
            .map_err(|_| "failed to update page flags")?
            .flush();
    }

    Ok(())
}

/// recursively frees the page tables starting from the given physical address and level, then frees the page table itself
unsafe fn page_table_free(table_phys: PhysAddr, level: u8) {
    log::trace!(
        "freeing page table at {:#x} (level {})",
        table_phys.as_u64(),
        level
    );

    let table =
        unsafe { &mut *phys_to_virt(table_phys).as_mut_ptr::<PageTable>() };
    let entry_limit = if level == 4 { 256 } else { 512 };

    for index in 0..entry_limit {
        let entry = &mut table[index];
        let flags = entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            continue;
        }

        if level > 1
            && !flags.contains(PageTableFlags::HUGE_PAGE)
            && let Ok(frame) = entry.frame()
        {
            let child_phys = frame.start_address();
            unsafe { page_table_free(child_phys, level - 1) };
            system::mem::pmm::free(child_phys.as_u64());
        }

        entry.set_unused();
    }
}

pub struct AddressSpace {
    pml4_phys: PhysAddr,
}

impl AddressSpace {
    pub fn new() -> Result<Self, &'static str> {
        let pml4_phys_addr = system::mem::pmm::alloc().ok_or("oom")?;
        let pml4_phys = PhysAddr::new(pml4_phys_addr);

        unsafe {
            let pml4_virt = phys_to_virt(pml4_phys).as_mut_ptr::<u8>();
            core::ptr::write_bytes(pml4_virt, 0, 4096); // zero out the new page table
        }

        let kernel_pml4_phys = PML4.lock().ok_or("VMM not initialized")?;

        unsafe {
            let kernel_pml4 = phys_to_virt(kernel_pml4_phys).as_ptr::<u64>();
            let new_pml4 = phys_to_virt(pml4_phys).as_mut_ptr::<u64>();

            // copy kernel mappings
            for i in 256..512 {
                let entry = kernel_pml4.add(i).read();
                new_pml4.add(i).write(entry);
            }
        }

        Ok(Self { pml4_phys })
    }

    /// returns the phys addr of the cr3
    pub fn cr3(&self) -> u64 { self.pml4_phys.as_u64() }

    /// maps a single page at the given virtual address to the given physical address with the specified flags
    pub fn map_page(
        &self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: PageTableFlags,
    ) -> Result<(), &'static str> {
        let page: Page<Size4KiB> = Page::containing_address(virt);
        let frame = PhysFrame::containing_address(phys);

        unsafe {
            let mut mapper = page_get_table_at(self.pml4_phys);
            let mut allocator = PMMFrameAllocator;

            match mapper.map_to(page, frame, flags, &mut allocator) {
                Ok(flush) => {
                    flush.ignore();
                },
                Err(e) => {
                    log::debug!(
                        "map_to failed for virt={:#x} phys={:#x}: {:?}",
                        virt.as_u64(),
                        phys.as_u64(),
                        e
                    );
                    return Err("failed to map page in address space");
                },
            }
        }

        Ok(())
    }

    /// maps a single page at the given virtual address to a newly allocated physical page with the specified flags
    pub fn map_page_alloc(
        &self,
        virt: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<PhysAddr, &'static str> {
        let phys_addr = system::mem::pmm::alloc().ok_or("oom")?;
        let phys = PhysAddr::new(phys_addr);

        unsafe {
            let virt_ptr = phys_to_virt(phys).as_mut_ptr::<u8>();
            core::ptr::write_bytes(virt_ptr, 0, 4096);
        }

        if let Err(e) = self.map_page(virt, phys, flags) {
            log::error!("failed to map page in address space: {}", e);
            system::mem::pmm::free(phys.as_u64());
            return Err(e);
        }

        Ok(phys)
    }

    /// unmaps the page at the given virtual address and returns the physical address that was mapped there
    pub fn unmap_page(&self, virt: VirtAddr) -> Result<PhysAddr, &'static str> {
        let page: Page<Size4KiB> = Page::containing_address(virt);

        unsafe {
            let mut mapper = page_get_table_at(self.pml4_phys);
            let (frame, flush) = mapper
                .unmap(page)
                .map_err(|_| "failed to unmap page in address space")?;
            flush.flush();
            Ok(frame.start_address())
        }
    }

    /// returns true if the given virtual address is mapped to a physical address, false otherwise
    pub fn is_mapped(&self, virt: VirtAddr) -> bool {
        unsafe {
            let mapper = page_get_table_at(self.pml4_phys);
            mapper.translate_addr(virt).is_some()
        }
    }

    /// returns the page table flags for the given virtual address, or an error if it's not mapped
    pub fn page_flags(
        &self,
        virt: VirtAddr,
    ) -> Result<PageTableFlags, &'static str> {
        let page: Page<Size4KiB> = Page::containing_address(virt);

        unsafe {
            let mapper = page_get_table_at(self.pml4_phys);
            match mapper.translate(page.start_address()) {
                x86_64::structures::paging::mapper::TranslateResult::Mapped {
                    flags,
                    ..
                } => Ok(flags),
                _ => Err("page not mapped"),
            }
        }
    }

    /// updates the page table flags for the given virtual address, returns an error if it's not mapped or if the update fails
    pub fn update_page_flags(
        &self,
        virt: VirtAddr,
        flags: PageTableFlags,
    ) -> Result<(), &'static str> {
        let page: Page<Size4KiB> = Page::containing_address(virt);

        unsafe {
            let mut mapper = page_get_table_at(self.pml4_phys);
            mapper
                .update_flags(page, flags)
                .map_err(|_| "failed to update page flags in address space")?
                .flush();
        }

        Ok(())
    }
}

// copying/zero
impl AddressSpace {
    /// zeros the given range of virtual addresses, returns an error if any page in the range is not mapped
    pub fn zero(&self, virt: VirtAddr, len: usize) -> Result<(), &'static str> {
        let mut offset = 0;

        while offset < len {
            let current_virt = VirtAddr::new(virt.as_u64() + offset as u64);
            let page_offset = (current_virt.as_u64() & 0xFFF) as usize;
            let bytes_in_page =
                core::cmp::min(4096 - page_offset, len - offset);

            let phys = unsafe {
                let mapper = page_get_table_at(self.pml4_phys);
                mapper.translate_addr(current_virt).ok_or("page not mapped")?
            };

            unsafe {
                let dest = phys_to_virt(phys).as_mut_ptr::<u8>();
                core::ptr::write_bytes(dest, 0, bytes_in_page);
            }

            offset += bytes_in_page;
        }

        Ok(())
    }

    /// writes the given data to the given virtual address, returns an error if any page in the range is not mapped
    pub fn write(
        &self,
        virt: VirtAddr,
        data: &[u8],
    ) -> Result<(), &'static str> {
        let mut offset = 0;

        while offset < data.len() {
            let current_virt = VirtAddr::new(virt.as_u64() + offset as u64);
            let page_offset = (current_virt.as_u64() & 0xFFF) as usize;
            let bytes_in_page =
                core::cmp::min(4096 - page_offset, data.len() - offset);

            let phys = unsafe {
                let mapper = page_get_table_at(self.pml4_phys);
                mapper.translate_addr(current_virt).ok_or("page not mapped")?
            };

            unsafe {
                let dest = phys_to_virt(phys).as_mut_ptr::<u8>();
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(offset),
                    dest,
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
    fn copy_user_pages_recursive(
        &self,
        dst: &AddressSpace,
        table_phys: PhysAddr,
        level: u8,
        base: u64,
    ) -> Result<(), &'static str> {
        let table = unsafe { &*phys_to_virt(table_phys).as_ptr::<PageTable>() };
        let entry_limit = if level == 4 { 256 } else { 512 };

        for index in 0..entry_limit {
            let entry = &table[index];
            let flags = entry.flags();

            if !flags.contains(PageTableFlags::PRESENT) {
                continue;
            }

            let level_shift = 12 + 9 * ((level as u64).saturating_sub(1));
            let entry_base = base + ((index as u64) << level_shift);

            if level == 1 {
                let src_phys =
                    entry.frame().map_err(|_| "invalid leaf page frame")?;

                let mut map_flags = PageTableFlags::PRESENT;
                map_flags |= flags
                    & (PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::WRITE_THROUGH
                        | PageTableFlags::NO_CACHE
                        | PageTableFlags::NO_EXECUTE);

                let dst_phys =
                    dst.map_page_alloc(VirtAddr::new(entry_base), map_flags)?;

                unsafe {
                    core::ptr::copy_nonoverlapping(
                        phys_to_virt(src_phys.start_address()).as_ptr::<u8>(),
                        phys_to_virt(dst_phys).as_mut_ptr::<u8>(),
                        4096,
                    );
                }
            } else {
                if flags.contains(PageTableFlags::HUGE_PAGE) {
                    return Err("huge pages are not supported for fork");
                }

                let next_table =
                    entry.frame().map_err(|_| "invalid page table frame")?;
                self.copy_user_pages_recursive(
                    dst,
                    next_table.start_address(),
                    level - 1,
                    entry_base,
                )?;
            }
        }

        Ok(())
    }

    /// creates a new address space with the same mappings as the current one for the user portion
    pub fn clone_user(&self) -> Result<Self, &'static str> {
        let dst = AddressSpace::new()?;
        self.copy_user_pages_recursive(&dst, self.pml4_phys, 4, 0)?;
        Ok(dst)
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        let pml4_phys = self.pml4_phys;
        let (current_pml4, _) = Cr3::read();

        if current_pml4.start_address() == pml4_phys {
            log::error!(
                "refusing to free active address space PML4 at {:#x}",
                pml4_phys.as_u64()
            );
            return;
        }

        if PML4.lock().as_ref().copied() == Some(pml4_phys) {
            log::error!(
                "refusing to free kernel address space PML4 at {:#x}",
                pml4_phys.as_u64()
            );
            return;
        }

        unsafe {
            page_table_free(pml4_phys, 4);
        }
        system::mem::pmm::free(pml4_phys.as_u64());
        log::trace!(
            "dropped address space and freed PML4 at {:#x}",
            pml4_phys.as_u64()
        );
    }
}
