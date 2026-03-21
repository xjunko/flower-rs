use x86_64::VirtAddr;
use x86_64::registers::model_specific::FsBase;
use x86_64::structures::paging::PageTableFlags;

use crate::debug;
use crate::system::syscalls::types::{SyscallError, SyscallFrame};
use crate::system::{self};

pub fn exit(_frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    system::proc::exit();
    unreachable!();
}

pub fn msleep(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let ms = frame.rdi;
    system::proc::sleep(ms);
    Ok(0)
}

pub fn write_fsbase(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let arg1 = frame.rdi;
    if let fsbase = VirtAddr::new(arg1)
        && system::mem::vmm::page_is_mapped(fsbase)
    {
        debug!("writing fsbase with value {:#x}", arg1);
        let fsbase = arg1;
        FsBase::write(VirtAddr::new(fsbase));
        Ok(0)
    } else {
        Ok(SyscallError::NotFound as u64)
    }
}

/// HACK: no fd implementation yet.
pub fn mmap(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    // start - r->rdi
    // size - r->rsi
    // prot - r->rdx
    // flags - r->r10
    // fd - r->r8
    // offset - r->r9

    let fd = frame.r8 as i64;
    let offset = frame.r9;
    let size = frame.rsi;
    let flags = frame.r10;

    debug!(
        "mmap: fd={}, offset={}, flags={}, size={}",
        fd, offset, flags, size
    );

    if let Some(mut proc) =
        system::proc::current().ok_or(SyscallError::Other)?.try_lock()
    {
        let heap_start = proc.user_heap_position;
        let heap_pages = (size + system::mem::PAGE_SIZE as u64 - 1)
            / system::mem::PAGE_SIZE as u64;

        let mut heap_ptr = heap_start;

        if fd != -1 {
            panic!("mmap with fd is not supported");
        } else {
            for _ in 0..heap_pages {
                proc.address_space.as_mut().unwrap().map_page_alloc(
                VirtAddr::new(heap_ptr),
                PageTableFlags::PRESENT
                    | PageTableFlags::USER_ACCESSIBLE
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_EXECUTE,
            ).map_err(|_| {
                debug!(
                    "mmap failed: could not map page at user heap position {:#x}",
                    proc.user_heap_position
                );
                SyscallError::NotFound
            })?;
                heap_ptr += system::mem::PAGE_SIZE as u64;
            }
            proc.user_heap_position = heap_ptr;
            return Ok(heap_start);
        }
    }

    Ok(SyscallError::NotFound as u64)
}
