use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::system::ToSyscallError;
use crate::system::mem::vmm;
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

    let current = system::proc::current()
        .ok_or(SyscallError::Other("no current process found".into()))?;
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
            log::debug!(
                "mmap: mapping fd {} at offset {} to user heap position {:#x} with size {}",
                fd,
                offset,
                proc.user_heap_position,
                size
            );

            log::debug!(
                "mmap: fd {} mmap returned data pointer {:#x}",
                fd,
                data as u64
            );

            for i in 0..heap_pages {
                let src_virt = VirtAddr::new(unsafe {
                    data.add(i as usize * arch::layout::PAGE_SIZE) as u64
                });
                let src_phys = vmm::virt_to_phys(src_virt).ok_or_else(|| {
                    log::error!(
                        "mmap failed: could not translate source virt {:#x} to phys",
                        src_virt.as_u64()
                    );
                    SyscallError::InvalidArgument
                })?;

                proc.address_space.as_mut().unwrap().map_page(
                    VirtAddr::new(heap_ptr),
                    src_phys,
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
            log::info!(
                "mmap: successfully mapped fd {} to user heap position {:#x} - {:#x}",
                fd,
                heap_start,
                proc.user_heap_position
            );
            Ok(heap_start)
        } else {
            log::error!("mmap failed for fd {}", fd);
            Err(result.err().unwrap().to_syscall_error())
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
        let current = system::proc::current()
            .ok_or(SyscallError::Other("no current process".into()))?;
        let mut proc = current.lock();

        if addr < proc.user_heap || end > proc.user_heap_position {
            log::error!(
                "munmap failed: address range {:#x} - {:#x} is out of bounds for user heap ({:#x} - {:#x})",
                addr,
                end,
                proc.user_heap,
                proc.user_heap_position
            );
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

            // NOTE: this might fuck me later
            if system::mem::pmm::is_usable_address(phys.as_u64()) {
                system::mem::pmm::free(phys.as_u64());
            } else {
                log::debug!(
                    "munmap: skipping free for non-usable physical page {:#x}",
                    phys.as_u64()
                );
            }
        }

        Ok(0)
    }
}
