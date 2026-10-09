use core::arch::asm;

use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::{
    FlagUpdateError, TranslateResult, UnmapError,
};
use x86_64::structures::paging::{Mapper, PhysFrame, Translate};
use x86_64::{PhysAddr, VirtAddr};

use crate::arch::MapFlags;
use crate::arch::paging::{
    KERNEL_HALF_FIRST_SLOT, KERNEL_ROOT, Root, copy_user_pages, free_table,
    from_hw, hhdm, map_raw, mapper, page_of, table_ptr, to_hw, zero_frame,
};

// impl starts here
pub fn hhdm_offset() -> u64 { hhdm() }

pub fn active_root() -> Root {
    let (frame, _) = Cr3::read();
    frame.start_address().as_u64()
}

pub fn set_active_root(root: Root) {
    let (_, flags) = Cr3::read();
    let frame = PhysFrame::containing_address(PhysAddr::new(root));
    unsafe { Cr3::write(frame, flags) };
}

pub fn root_new() -> Result<Root, &'static str> {
    let kernel_root = *KERNEL_ROOT.get().ok_or("paging not initialized")?;
    let root = crate::system::mem::pmm::alloc().ok_or("oom")?;

    unsafe {
        zero_frame(root);

        let kernel_pml4 = &*table_ptr(kernel_root);
        let new_pml4 = &mut *table_ptr(root);

        // share the kernel half
        for i in KERNEL_HALF_FIRST_SLOT..512 {
            new_pml4[i] = kernel_pml4[i].clone();
        }
    }

    Ok(root)
}

pub fn root_free(root: Root) {
    unsafe { free_table(root, 4) };
    crate::system::mem::pmm::free(root);
}

pub fn root_phys(root: Root) -> u64 { root }

pub fn map(
    root: Root,
    virt: u64,
    phys: u64,
    flags: MapFlags,
) -> Result<(), &'static str> {
    map_raw(root, virt, phys, to_hw(flags))
}

pub fn unmap(root: Root, virt: u64) -> Result<u64, &'static str> {
    let page = page_of(virt)?;

    unsafe {
        let (frame, flush) = mapper(root).unmap(page).map_err(|e| match e {
            UnmapError::PageNotMapped => "page not mapped",
            UnmapError::ParentEntryHugePage => "parent entry is a huge page",
            UnmapError::InvalidFrameAddress(_) => "invalid frame address",
        })?;

        // flushing is the caller's job (see `tlb_flush`)
        flush.ignore();
        Ok(frame.start_address().as_u64())
    }
}

pub fn protect(
    root: Root,
    virt: u64,
    flags: MapFlags,
) -> Result<(), &'static str> {
    let page = page_of(virt)?;

    unsafe {
        mapper(root)
            .update_flags(page, to_hw(flags))
            .map_err(|e| match e {
                FlagUpdateError::PageNotMapped => "page not mapped",
                FlagUpdateError::ParentEntryHugePage => {
                    "parent entry is a huge page"
                },
            })?
            .ignore();
    }

    Ok(())
}

pub fn translate(root: Root, virt: u64) -> Option<(u64, MapFlags)> {
    let virt = VirtAddr::try_new(virt).ok()?;

    let mapper = unsafe { mapper(root) };
    match mapper.translate(virt) {
        TranslateResult::Mapped { frame, offset, flags } => {
            Some((frame.start_address().as_u64() + offset, from_hw(flags)))
        },
        _ => None,
    }
}

pub fn tlb_flush(virt: u64) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) virt,
            options(nostack, preserves_flags)
        );
    }
}

pub fn tlb_flush_all() { x86_64::instructions::tlb::flush_all(); }

pub fn clone_user(root: Root) -> Result<Root, &'static str> {
    let dst = root_new()?;

    if let Err(e) = unsafe { copy_user_pages(dst, root, 4, 0) } {
        root_free(dst);
        return Err(e);
    }

    Ok(dst)
}
