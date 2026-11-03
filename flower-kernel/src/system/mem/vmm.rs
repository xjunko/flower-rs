use spin::Mutex;

use crate::arch::addr::{PhysAddr, VirtAddr};
use crate::arch::generic::paging::MapFlags;
use crate::arch::layout::{KERNEL_VALLOC_END, KERNEL_VALLOC_START};
use crate::arch::{Paging, Processor};
use crate::system::mem;

type Root = <Processor as Paging>::Root;

const PAGE_SIZE: usize = <Processor as Paging>::PAGE_SIZE;
const PAGE_MASK: u64 = PAGE_SIZE as u64 - 1;

static KERNEL_SPACE: Mutex<Option<AddressSpace>> = Mutex::new(None);

pub fn install() {
    debug_assert!(
        KERNEL_VALLOC_START >= Processor::KERNEL_HALF_START,
        "kernel valloc range must live in the kernel half"
    );

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
}

impl AddressSpace {
    /// creates a new address space. the kernel half is shared with the
    /// kernel address space by `Paging::root_new`.
    pub fn new() -> Result<Self, &'static str> {
        let root = Processor::paging_root_new()?;
        Ok(Self { root, owned: true })
    }

    /// wraps an existing root table into an AddressSpace (does not own it)
    pub fn wrap(root: Root) -> Self { Self { root, owned: false } }

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
    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        Processor::paging_translate(self.root, virt).map(|(phys, _)| phys)
    }

    /// returns true if the given virtual address is mapped in this address space
    pub fn is_mapped(&self, virt: VirtAddr) -> bool {
        self.translate(virt).is_some()
    }

    /// translates a virtual address of the currently active address space to a physical one
    pub fn virt_to_phys(virt: VirtAddr) -> Option<PhysAddr> {
        let hhdm = VirtAddr::new(Processor::paging_hhdm());

        // can safely short circuit here
        if let Some(max_phys) = mem::pmm::max_phys_address()
            && let Some(hhdm_end) = hhdm.as_u64().checked_add(max_phys)
            && virt >= hhdm
            && virt.as_u64() < hhdm_end
        {
            return Some(PhysAddr::new(virt - hhdm));
        }

        Self::current().translate(virt)
    }

    /// returns a virtual address for the given physical address in the HHDM
    pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
        VirtAddr::new(phys.as_u64() + Processor::paging_hhdm())
    }
}

// mapping/unmapping
impl AddressSpace {
    /// maps a single page at the given virtual address to the given physical address with the specified flags
    pub fn map_page(
        &self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let virt = VirtAddr::new(virt.as_u64() & !PAGE_MASK);
        let phys = PhysAddr::new(phys.as_u64() & !PAGE_MASK);

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

        // a fresh mapping needs no flush on x86, but other archs (or
        // replacing a not-present entry that was cached) may.
        self.flush(virt);
        Ok(())
    }

    /// maps a single page at the given virtual address to a newly allocated, zeroed physical page
    pub fn map_page_alloc(
        &self,
        virt: VirtAddr,
        flags: MapFlags,
    ) -> Result<PhysAddr, &'static str> {
        let phys = PhysAddr::new(mem::pmm::alloc().ok_or("oom")?);

        unsafe {
            core::ptr::write_bytes(
                Self::phys_to_virt(phys).as_mut_ptr::<u8>(),
                0,
                PAGE_SIZE,
            );
        }

        if let Err(e) = self.map_page(virt, phys, flags) {
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
        virt: VirtAddr,
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

                // unmap everything we mapped before the failing page
                let mut done = start;
                while done < page {
                    if let Ok(phys) = self.unmap_page(done) {
                        mem::pmm::free(phys.as_u64());
                    }
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

    /// unmaps the page at the given virtual address and returns the physical address that was mapped there
    pub fn unmap_page(&self, virt: VirtAddr) -> Result<PhysAddr, &'static str> {
        let virt = VirtAddr::new(virt.as_u64() & !PAGE_MASK);
        let phys = Processor::paging_unmap(self.root, virt)?;
        self.flush(virt);
        Ok(phys)
    }

    /// unmaps a range of virtual addresses and frees the physical pages that were mapped there
    pub fn unmap_range(
        &self,
        virt: VirtAddr,
        size: usize,
    ) -> Result<(), &'static str> {
        let (start, end) = Self::page_span(virt, size)?;

        let mut page = start;
        loop {
            let phys = self.unmap_page(page)?;
            mem::pmm::free(phys.as_u64());

            if page == end {
                break;
            }
            page += PAGE_SIZE as u64;
        }

        Ok(())
    }

    /// returns the page flags for the given virtual address, or an error if it's not mapped
    pub fn page_flags(&self, virt: VirtAddr) -> Result<MapFlags, &'static str> {
        Processor::paging_translate(
            self.root,
            VirtAddr::new(virt.as_u64() & !PAGE_MASK),
        )
        .map(|(_, flags)| flags)
        .ok_or("page not mapped")
    }

    /// updates the page flags for the given virtual address, returns an error if it's not mapped or if the update fails
    pub fn update_page_flags(
        &self,
        virt: VirtAddr,
        flags: MapFlags,
    ) -> Result<(), &'static str> {
        let virt = VirtAddr::new(virt.as_u64() & !PAGE_MASK);
        Processor::paging_update(self.root, virt, flags)?;
        self.flush(virt);
        Ok(())
    }
}

// vmem alloc
static KERNEL_VALLOC_NEXT: Mutex<VirtAddr> =
    Mutex::new(VirtAddr::new(KERNEL_VALLOC_START));

impl AddressSpace {
    /// reserves a range of virtual addresses in the kernel address space, returns an error if the range is exhausted
    pub fn reserve_virt(size: usize) -> Result<VirtAddr, &'static str> {
        let pages = size.div_ceil(PAGE_SIZE) as u64;
        let bytes = pages * PAGE_SIZE as u64;

        let mut next = KERNEL_VALLOC_NEXT.lock();

        let start = *next;
        let end = VirtAddr::new(
            start.as_u64().checked_add(bytes).ok_or("valloc size overflow")?,
        );
        if end > VirtAddr::new(KERNEL_VALLOC_END) {
            return Err("valloc space exhausted");
        }
        *next = end;
        Ok(start)
    }

    /// allocates `size` bytes of memory and returns a virtual address
    pub fn alloc_virt(
        &self,
        size: usize,
        flags: MapFlags,
    ) -> Result<VirtAddr, &'static str> {
        if size == 0 {
            return Err("invalid size");
        }

        let virt = Self::reserve_virt(size)?;
        if let Err(e) = self.map_range_alloc(virt, size, flags) {
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

    /// frees a range of virtual addresses, used along alloc_virt
    pub fn free_virt(
        &self,
        virt: VirtAddr,
        size: usize,
    ) -> Result<(), &'static str> {
        self.unmap_range(virt, size)
    }
}

// utils
impl AddressSpace {
    /// returns the first and last page (inclusive) touched by `[virt, virt + size)`
    fn page_span(
        virt: VirtAddr,
        size: usize,
    ) -> Result<(VirtAddr, VirtAddr), &'static str> {
        if size == 0 {
            return Err("invalid size");
        }

        let last = virt
            .as_u64()
            .checked_add(size as u64 - 1)
            .ok_or("address range overflow")?;
        Ok((
            VirtAddr::new(virt.as_u64() & !PAGE_MASK),
            VirtAddr::new(last & !PAGE_MASK),
        ))
    }

    /// flushes the tlb entry for `virt` if this address space can be cached on this cpu.
    /// the kernel half is shared between every address space, so it is always flushed.
    fn flush(&self, virt: VirtAddr) {
        if virt >= VirtAddr::new(Processor::KERNEL_HALF_START)
            || self.root == Processor::paging_root_active()
        {
            Processor::paging_flush(virt);
        }
    }

    /// translates a (possibly unaligned) virtual address to its physical address
    fn phys_of(&self, virt: VirtAddr) -> Result<PhysAddr, &'static str> {
        let (phys, _) = Processor::paging_translate(
            self.root,
            VirtAddr::new(virt.as_u64() & !PAGE_MASK),
        )
        .ok_or("page not mapped")?;
        Ok(phys + (virt.as_u64() & PAGE_MASK))
    }
}

// copying/zero
impl AddressSpace {
    /// zeros the given range of virtual addresses, returns an error if any page in the range is not mapped
    pub fn zero(&self, virt: VirtAddr, len: usize) -> Result<(), &'static str> {
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
        virt: VirtAddr,
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
    /// creates a new address space with a copy of the user half of this one
    pub fn clone_user(&self) -> Result<Self, &'static str> {
        let root = Processor::paging_clone_user(self.root)?;
        Ok(Self { root, owned: true })
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
