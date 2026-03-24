use x86_64::structures::paging::PageTableFlags;
use x86_64::{PhysAddr, VirtAddr};

use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallError;
use crate::system::vfs::{FdKind, VFSError};
use crate::{arch, system};

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

    log::debug!(
        "mmap: fd={}, offset={}, flags={}, size={}",
        fd,
        offset,
        flags,
        size
    );

    if size == 0 {
        return Err(SyscallError::InvalidArgument);
    }

    let current = system::proc::current().ok_or(SyscallError::Other)?;
    let mut proc = current.lock();

    let heap_start = proc.user_heap_position;
    let heap_pages = (size + arch::layout::PAGE_SIZE as u64 - 1)
        / arch::layout::PAGE_SIZE as u64;

    let mut heap_ptr = heap_start;

    if fd != -1 {
        let result =
            proc.with_fd_table(|table| match table.get(fd as usize)? {
                FdKind::File(file) => file.mmap(size as usize, 0, 0),
                _ => Err(VFSError::Unsupported),
            });

        if let Ok(data) = result {
            for i in 0..heap_pages {
                proc.address_space.as_mut().unwrap().map_page(
                    VirtAddr::new(heap_ptr),
                    PhysAddr::new(unsafe {
                        data.add(i as usize * arch::layout::PAGE_SIZE) as u64
                    }),
                    PageTableFlags::PRESENT
                        | PageTableFlags::USER_ACCESSIBLE
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::NO_EXECUTE,
                ).map_err(|_| {
                    log::debug!(
                        "mmap failed: could not map page at user heap position {:#x}",
                        proc.user_heap_position
                    );
                    SyscallError::InvalidArgument
                })?;

                heap_ptr += arch::layout::PAGE_SIZE as u64;
            }
            proc.user_heap_position = heap_ptr;
            Ok(heap_start)
        } else {
            log::error!("mmap failed for fd {}: {:?}", fd, result.err());
            Err(SyscallError::BadFileDescriptor)
        }
    } else {
        for _ in 0..heap_pages {
            proc.address_space.as_mut().unwrap().map_page_alloc(
                VirtAddr::new(heap_ptr),
                PageTableFlags::PRESENT
                    | PageTableFlags::USER_ACCESSIBLE
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_EXECUTE,
            ).map_err(|_| {
                log::debug!(
                    "mmap failed: could not map page at user heap position {:#x}",
                    proc.user_heap_position
                );
                SyscallError::InvalidArgument
            })?;
            heap_ptr += arch::layout::PAGE_SIZE as u64;
        }

        proc.user_heap_position = heap_ptr;
        Ok(heap_start)
    }
}

pub fn munmap(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let addr = frame.rdi;
    let base = addr & !(arch::layout::PAGE_SIZE as u64 - 1);
    let size = frame.rsi;

    if size == 0 {
        return Err(SyscallError::InvalidArgument);
    }

    let end = addr.checked_add(size).ok_or(SyscallError::InvalidArgument)?;

    let pages = (size + arch::layout::PAGE_SIZE as u64 - 1)
        / arch::layout::PAGE_SIZE as u64;

    log::debug!("munmap: addr={:#x}, size={}, pages={}", addr, size, pages);

    {
        let current = system::proc::current().ok_or(SyscallError::Other)?;
        let mut proc = current.lock();

        if addr < proc.user_heap || end > proc.user_heap_position {
            return Err(SyscallError::InvalidArgument);
        }

        for i in 0..pages {
            let page_addr = base + i * arch::layout::PAGE_SIZE as u64;
            let phys = proc.address_space.as_mut().unwrap().unmap_page(VirtAddr::new(page_addr)).map_err(|_| {
                log::error!(
                    "munmap failed: could not unmap page at user heap position {:#x}",
                    page_addr
                );
                SyscallError::InvalidArgument
            })?;
            system::mem::pmm::free(phys.as_u64());
        }

        Ok(0)
    }
}
