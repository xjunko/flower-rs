use spin::Once;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
    PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

mod api;
pub use api::*;

use crate::arch::MapFlags;
use crate::arch::x86_64::layout::{KERNEL_VALLOC_END, KERNEL_VALLOC_START};

const KERNEL_HALF_FIRST_SLOT: usize = 256;

static HHDM: Once<u64> = Once::new();
static KERNEL_ROOT: Once<u64> = Once::new();

struct PMMFrameAllocator;
unsafe impl FrameAllocator<Size4KiB> for PMMFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let addr = crate::system::mem::pmm::alloc()?;
        Some(PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

pub fn install() {
    HHDM.call_once(|| {
        crate::boot::limine::HHDM_REQUEST.response().expect("no hhdm").offset
    });

    let (pml4_frame, _) = Cr3::read();
    let kernel_root = pml4_frame.start_address().as_u64();
    KERNEL_ROOT.call_once(|| kernel_root);

    log::debug!("pre-populating kernel address space with existing mappings");

    let first_idx = usize::from(VirtAddr::new(KERNEL_VALLOC_START).p4_index());
    let last_idx = usize::from(VirtAddr::new(KERNEL_VALLOC_END).p4_index());
    debug_assert!(
        first_idx >= KERNEL_HALF_FIRST_SLOT,
        "kernel address space should start at 0xFFFF800000000000"
    );

    let pml4 = unsafe { &mut *table_ptr(kernel_root) };

    let mut created = 0;
    for i in first_idx..=last_idx {
        if !pml4[i].is_unused() {
            continue;
        }

        let phys = crate::system::mem::pmm::alloc()
            .expect("oom while populating kernel pml4");
        unsafe { zero_frame(phys) };

        pml4[i].set_addr(
            PhysAddr::new(phys),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );
        created += 1;
    }

    log::debug!(
        "kernel pml4: slots {}..={} ready ({} newly created)",
        first_idx,
        last_idx,
        created
    );
}

pub fn hhdm() -> u64 { *HHDM.get().expect("paging not initialized") }

pub fn table_ptr(phys: u64) -> *mut PageTable {
    (phys + hhdm()) as *mut PageTable
}

unsafe fn zero_frame(phys: u64) {
    unsafe {
        core::ptr::write_bytes(
            (phys + hhdm()) as *mut u8,
            0,
            crate::arch::layout::PAGE_SIZE,
        );
    }
}

unsafe fn mapper(root: u64) -> OffsetPageTable<'static> {
    let pml4 = unsafe { &mut *table_ptr(root) };
    unsafe { OffsetPageTable::new(pml4, VirtAddr::new(hhdm())) }
}

fn page_of(virt: u64) -> Result<Page<Size4KiB>, &'static str> {
    let virt = VirtAddr::try_new(virt).map_err(|_| "non-canonical address")?;
    Ok(Page::containing_address(virt))
}

fn map_err(e: MapToError<Size4KiB>) -> &'static str {
    match e {
        MapToError::FrameAllocationFailed => "oom",
        MapToError::ParentEntryHugePage => "parent entry is a huge page",
        MapToError::PageAlreadyMapped(_) => "page already mapped",
    }
}

/// generic flags -> hardware flags.
fn to_hw(flags: MapFlags) -> PageTableFlags {
    let mut hw = PageTableFlags::PRESENT;

    if flags.contains(MapFlags::WRITE) {
        hw |= PageTableFlags::WRITABLE;
    }
    if !flags.contains(MapFlags::EXEC) {
        hw |= PageTableFlags::NO_EXECUTE;
    }
    if flags.contains(MapFlags::USER) {
        hw |= PageTableFlags::USER_ACCESSIBLE;
    }
    if flags.contains(MapFlags::WRITE_THROUGH) {
        hw |= PageTableFlags::WRITE_THROUGH;
    }
    if flags.contains(MapFlags::NO_CACHE) {
        hw |= PageTableFlags::NO_CACHE;
    }

    hw
}

/// hardware flags -> generic flags.
fn from_hw(hw: PageTableFlags) -> MapFlags {
    let mut flags = MapFlags::empty();

    if hw.contains(PageTableFlags::WRITABLE) {
        flags |= MapFlags::WRITE;
    }
    if !hw.contains(PageTableFlags::NO_EXECUTE) {
        flags |= MapFlags::EXEC;
    }
    if hw.contains(PageTableFlags::USER_ACCESSIBLE) {
        flags |= MapFlags::USER;
    }
    if hw.contains(PageTableFlags::WRITE_THROUGH) {
        flags |= MapFlags::WRITE_THROUGH;
    }
    if hw.contains(PageTableFlags::NO_CACHE) {
        flags |= MapFlags::NO_CACHE;
    }

    flags
}

/// maps `virt` -> `phys` with raw hardware flags. no tlb maintenance.
fn map_raw(
    root: u64,
    virt: u64,
    phys: u64,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    let page = page_of(virt)?;
    let phys =
        PhysAddr::try_new(phys).map_err(|_| "invalid physical address")?;
    let frame = PhysFrame::containing_address(phys);

    unsafe {
        mapper(root)
            .map_to(page, frame, flags, &mut PMMFrameAllocator)
            .map_err(map_err)?
            .ignore();
    }

    Ok(())
}

/// maps `virt` to a freshly allocated, zeroed frame and returns the frame.
fn map_alloc(
    root: u64,
    virt: u64,
    flags: PageTableFlags,
) -> Result<u64, &'static str> {
    let phys = crate::system::mem::pmm::alloc().ok_or("oom")?;
    unsafe { zero_frame(phys) };

    if let Err(e) = map_raw(root, virt, phys, flags) {
        crate::system::mem::pmm::free(phys);
        return Err(e);
    }

    Ok(phys)
}

/// frees the user half of a table hierarchy (and the tables themselves).
/// does not free `table_phys` itself.
unsafe fn free_table(table_phys: u64, level: u8) {
    log::trace!("freeing page table at {:#x} (level {})", table_phys, level);

    let table = unsafe { &mut *table_ptr(table_phys) };
    let entry_limit = if level == 4 { KERNEL_HALF_FIRST_SLOT } else { 512 };

    for index in 0..entry_limit {
        let entry = &mut table[index];
        let flags = entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            continue;
        }

        if level > 1 {
            // huge pages are never created for user space (clone_user
            // refuses them), so there is nothing to free for them here.
            if !flags.contains(PageTableFlags::HUGE_PAGE) {
                let child = entry.addr().as_u64();
                unsafe { free_table(child, level - 1) };
                crate::system::mem::pmm::free(child);
            }
        } else {
            crate::system::mem::pmm::free(entry.addr().as_u64());
        }

        entry.set_unused();
    }
}

/// deep copies the user half of the table at `table_phys` into `dst_root`.
unsafe fn copy_user_pages(
    dst_root: u64,
    table_phys: u64,
    level: u8,
    base: u64,
) -> Result<(), &'static str> {
    let table = unsafe { &*table_ptr(table_phys) };
    let entry_limit = if level == 4 { KERNEL_HALF_FIRST_SLOT } else { 512 };

    for index in 0..entry_limit {
        let entry = &table[index];
        let flags = entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            continue;
        }

        let level_shift = 12 + 9 * (level as u64 - 1);
        let entry_base = base + ((index as u64) << level_shift);

        if level == 1 {
            let map_flags = PageTableFlags::PRESENT
                | (flags
                    & (PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::WRITE_THROUGH
                        | PageTableFlags::NO_CACHE
                        | PageTableFlags::NO_EXECUTE));

            let dst_phys = map_alloc(dst_root, entry_base, map_flags)?;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    (entry.addr().as_u64() + hhdm()) as *const u8,
                    (dst_phys + hhdm()) as *mut u8,
                    crate::arch::layout::PAGE_SIZE,
                );
            }
        } else {
            if flags.contains(PageTableFlags::HUGE_PAGE) {
                return Err("huge pages are not supported for fork");
            }

            unsafe {
                copy_user_pages(
                    dst_root,
                    entry.addr().as_u64(),
                    level - 1,
                    entry_base,
                )?;
            }
        }
    }

    Ok(())
}

// top part of the code is so messy so this goes down here
pub type Root = u64;
