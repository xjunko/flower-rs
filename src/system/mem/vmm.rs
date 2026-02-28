use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB, Translate,
    },
};

use crate::{debug, error, info, system};

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
    *HHDM.lock() = Some(
        crate::boot::limine::HHDM_REQUEST
            .get_response()
            .unwrap()
            .offset(),
    );

    let (pml4_frame, _) = Cr3::read();
    *PML4.lock() = Some(pml4_frame.start_address());

    info!("VMM installed.");
    info!("PML4 physical address: {:#x}", pml4_frame.start_address());
}

fn hhdm() -> u64 {
    HHDM.lock().expect("no hhdm")
}

pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    VirtAddr::new(phys.as_u64() + hhdm())
}

pub fn virt_to_phys(virt: VirtAddr) -> Option<PhysAddr> {
    let hhdm = hhdm();

    if let Some(max_phys) = system::mem::pmm::max_phys_address()
        && let Some(hhdm_end) = hhdm.checked_add(max_phys)
        && virt.as_u64() >= hhdm
        && virt.as_u64() < hhdm_end
    {
        return Some(PhysAddr::new(virt.as_u64() - hhdm));
    }

    debug!(
        "virt_to_phys: address {:#x} is below hhdm {:#x}, translation needed",
        virt.as_u64(),
        hhdm
    );

    unsafe {
        let mapper = page_get_current_table();
        mapper.translate_addr(virt)
    }
}

unsafe fn page_get_current_table() -> OffsetPageTable<'static> {
    let (pml4_frame, _) = Cr3::read();
    let pml4_virt = phys_to_virt(pml4_frame.start_address());
    let pml4: &'static mut PageTable = unsafe { &mut *pml4_virt.as_mut_ptr() };
    unsafe { OffsetPageTable::new(pml4, VirtAddr::new(hhdm())) }
}

unsafe fn page_get_table_at(pml4_phys: PhysAddr) -> OffsetPageTable<'static> {
    let pml4_virt = phys_to_virt(pml4_phys);
    let pml4: &'static mut PageTable = unsafe { &mut *pml4_virt.as_mut_ptr() };
    unsafe { OffsetPageTable::new(pml4, VirtAddr::new(hhdm())) }
}

pub fn page_map(virt: VirtAddr, phys: PhysAddr, flags: PageTableFlags) -> Result<(), &'static str> {
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

pub fn page_map_alloc(virt: VirtAddr, flags: PageTableFlags) -> Result<PhysAddr, &'static str> {
    let phys_addr = system::mem::pmm::alloc().ok_or("oom")?;
    let phys = PhysAddr::new(phys_addr);

    unsafe {
        let virt_ptr = phys_to_virt(phys).as_mut_ptr::<u8>();
        core::ptr::write_bytes(virt_ptr, 0, 4096);
    }

    if let Err(e) = page_map(virt, phys, flags) {
        error!("Failed to map page: {}", e);
        system::mem::pmm::free(phys.as_u64());
        return Err(e);
    }

    Ok(phys)
}

pub fn page_unmap(virt: VirtAddr) -> Result<PhysAddr, &'static str> {
    debug!("unmapping page at virt {:#x}", virt.as_u64(),);
    let page: Page<Size4KiB> = Page::containing_address(virt);

    unsafe {
        let mut mapper = page_get_current_table();
        let (frame, flush) = mapper.unmap(page).map_err(|_| "failed to unmap page")?;
        flush.flush();
        Ok(frame.start_address())
    }
}

pub fn page_is_mapped(virt: VirtAddr) -> bool {
    debug!("checking if page at virt {:#x} is mapped", virt.as_u64(),);
    unsafe {
        let mapper = page_get_current_table();
        mapper.translate_addr(virt).is_some()
    }
}

unsafe fn page_table_free(table_phys: PhysAddr, level: u8) {
    debug!(
        "freeing page table at {:#x} (level {})",
        table_phys.as_u64(),
        level
    );

    let table = unsafe { &mut *phys_to_virt(table_phys).as_mut_ptr::<PageTable>() };
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

    pub fn cr3(&self) -> u64 {
        self.pml4_phys.as_u64()
    }

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

            mapper
                .map_to(page, frame, flags, &mut allocator)
                .map_err(|_| "failed to map page in address space")?
                .ignore();
        }

        Ok(())
    }

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
            error!("failed to map page in address space: {}", e);
            system::mem::pmm::free(phys.as_u64());
            return Err(e);
        }

        Ok(phys)
    }

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

    pub fn is_mapped(&self, virt: VirtAddr) -> bool {
        unsafe {
            let mapper = page_get_table_at(self.pml4_phys);
            mapper.translate_addr(virt).is_some()
        }
    }
}

// copying/zero
impl AddressSpace {
    pub fn zero(&self, virt: VirtAddr, len: usize) -> Result<(), &'static str> {
        let mut offset = 0;

        while offset < len {
            let current_virt = VirtAddr::new(virt.as_u64() + offset as u64);
            let page_offset = (current_virt.as_u64() & 0xFFF) as usize;
            let bytes_in_page = core::cmp::min(4096 - page_offset, len - offset);

            let phys = unsafe {
                let mapper = page_get_table_at(self.pml4_phys);
                mapper
                    .translate_addr(current_virt)
                    .ok_or("page not mapped")?
            };

            unsafe {
                let dest = phys_to_virt(phys).as_mut_ptr::<u8>();
                core::ptr::write_bytes(dest, 0, bytes_in_page);
            }

            offset += bytes_in_page;
        }

        Ok(())
    }

    pub fn write(&self, virt: VirtAddr, data: &[u8]) -> Result<(), &'static str> {
        let mut offset = 0;

        while offset < data.len() {
            let current_virt = VirtAddr::new(virt.as_u64() + offset as u64);
            let page_offset = (current_virt.as_u64() & 0xFFF) as usize;
            let bytes_in_page = core::cmp::min(4096 - page_offset, data.len() - offset);

            let phys = unsafe {
                let mapper = page_get_table_at(self.pml4_phys);
                mapper
                    .translate_addr(current_virt)
                    .ok_or("page not mapped")?
            };

            unsafe {
                let dest = phys_to_virt(phys).as_mut_ptr::<u8>();
                core::ptr::copy_nonoverlapping(data.as_ptr().add(offset), dest, bytes_in_page);
            }

            offset += bytes_in_page;
        }

        Ok(())
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        let pml4_phys = self.pml4_phys;
        let (current_pml4, _) = Cr3::read();

        if current_pml4.start_address() == pml4_phys {
            error!(
                "refusing to free active address space PML4 at {:#x}",
                pml4_phys.as_u64()
            );
            return;
        }

        if PML4.lock().as_ref().copied() == Some(pml4_phys) {
            error!(
                "refusing to free kernel address space PML4 at {:#x}",
                pml4_phys.as_u64()
            );
            return;
        }

        unsafe {
            page_table_free(pml4_phys, 4);
        }
        system::mem::pmm::free(pml4_phys.as_u64());
        debug!(
            "dropped address space and freed PML4 at {:#x}",
            pml4_phys.as_u64()
        );
    }
}
