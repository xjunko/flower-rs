use core::ffi::c_int;

use flower_mono::mmap::{
    MAP_ANONYMOUS, MAP_PRIVATE, MAP_SHARED, PROT_EXEC, PROT_NONE, PROT_WRITE,
};
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;

use crate::arch::x86_64::layout::PAGE_SIZE;
use crate::memory::{self, vmm};
use crate::system::ToSyscallError;
use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallError;
use crate::system::vfs::{FdKind, VFSError};
use crate::{arch, system};

#[allow(clippy::manual_is_multiple_of)]
pub fn mmap(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let addr = frame.rdi;
    let size = frame.rsi;
    let prot = frame.rdx;
    let flags = frame.r10;
    let fd = frame.r8 as i64;
    let offset = frame.r9;

    log::debug!(
        "mmap: fd={}, offset={}, flags={}, size={}",
        fd,
        offset,
        flags,
        size
    );

    if size == 0 {
        log::error!("mmap failed: size is 0");
        return Err(SyscallError::InvalidArgument);
    }

    if offset % arch::x86_64::layout::PAGE_SIZE as u64 != 0 {
        log::error!("mmap failed: offset is not page-aligned");
        return Err(SyscallError::InvalidArgument);
    }

    let has_shared = flags & MAP_SHARED != 0;
    let has_private = flags & MAP_PRIVATE != 0;

    if fd != -1 && flags & MAP_ANONYMOUS == 0 {
        if !has_shared && !has_private {
            log::error!(
                "mmap failed: file mapping requires MAP_SHARED or MAP_PRIVATE"
            );
            return Err(SyscallError::InvalidArgument);
        }

        if has_shared && has_private {
            log::error!(
                "mmap failed: MAP_SHARED and MAP_PRIVATE cannot both be set"
            );
            return Err(SyscallError::InvalidArgument);
        }
    }
    let current = system::proc::current()
        .ok_or(SyscallError::Other("no current process found".into()))?;
    let mut proc = current.lock();

    let heap_start = if addr == 0 {
        proc.user_heap_position()
    } else {
        addr & !(arch::x86_64::layout::PAGE_SIZE as u64 - 1)
    };
    let heap_pages = (size + arch::x86_64::layout::PAGE_SIZE as u64 - 1)
        / arch::x86_64::layout::PAGE_SIZE as u64;

    let mut heap_ptr = heap_start;
    let mut page_flags = PageTableFlags::PRESENT;

    if prot != PROT_NONE {
        page_flags |= PageTableFlags::USER_ACCESSIBLE;
    }

    if prot & PROT_WRITE != 0 {
        page_flags |= PageTableFlags::WRITABLE;
    }

    if prot & PROT_EXEC == 0 {
        page_flags |= PageTableFlags::NO_EXECUTE;
    }

    if fd != -1 && flags & MAP_ANONYMOUS == 0 {
        let result =
            proc.with_fd_table(|table| match table.get(fd as usize)? {
                FdKind::File(file) => file.mmap(
                    size as usize,
                    prot as c_int,
                    flags as c_int,
                    offset,
                ),
                _ => Err(VFSError::Unsupported),
            });

        if let Ok(data) = result {
            log::debug!(
                "mmap: mapping fd {} at offset {} to user heap position {:#x} with size {}",
                fd,
                offset,
                proc.user_heap_position(),
                size
            );

            log::debug!(
                "mmap: fd {} mmap returned data pointer {:#x}",
                fd,
                data as u64
            );

            for i in 0..heap_pages {
                let src_virt = VirtAddr::new(unsafe {
                    data.add(i as usize * arch::x86_64::layout::PAGE_SIZE)
                        as u64
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
                    page_flags,
                ).map_err(|_| {
                    log::debug!(
                        "mmap failed: could not map page at user heap position {:#x}",
                        proc.user_heap_position()
                    );
                    SyscallError::InvalidArgument
                })?;

                heap_ptr += arch::x86_64::layout::PAGE_SIZE as u64;
            }
            if heap_ptr > proc.user_heap_position() {
                proc.set_user_heap_position(heap_ptr);
            }
            log::debug!(
                "mmap: successfully mapped fd {} to user heap position {:#x} - {:#x}",
                fd,
                heap_start,
                proc.user_heap_position()
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
                page_flags,
            ).map_err(|_| {
                log::debug!(
                    "mmap failed: could not map page at user heap position {:#x}",
                    proc.user_heap_position()
                );
                SyscallError::InvalidArgument
            })?;
            heap_ptr += arch::x86_64::layout::PAGE_SIZE as u64;
        }

        if heap_ptr > proc.user_heap_position() {
            proc.set_user_heap_position(heap_ptr);
        }
        Ok(heap_start)
    }
}

pub fn munmap(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let addr = frame.rdi;
    let base = addr & !(arch::x86_64::layout::PAGE_SIZE as u64 - 1);
    let size = frame.rsi;

    if size == 0 {
        return Err(SyscallError::InvalidArgument);
    }

    let end = addr.checked_add(size).ok_or(SyscallError::InvalidArgument)?;

    let pages = (size + arch::x86_64::layout::PAGE_SIZE as u64 - 1)
        / arch::x86_64::layout::PAGE_SIZE as u64;

    log::debug!("munmap: addr={:#x}, size={}, pages={}", addr, size, pages);

    {
        let current = system::proc::current()
            .ok_or(SyscallError::Other("no current process".into()))?;
        let mut proc = current.lock();

        if addr < proc.user_heap() || end > proc.user_heap_position() {
            log::error!(
                "munmap failed: address range {:#x} - {:#x} is out of bounds for user heap ({:#x} - {:#x})",
                addr,
                end,
                proc.user_heap(),
                proc.user_heap_position()
            );
            return Err(SyscallError::InvalidArgument);
        }

        for i in 0..pages {
            let page_addr = base + i * arch::x86_64::layout::PAGE_SIZE as u64;
            let phys = proc.address_space.as_mut().unwrap().unmap_page(VirtAddr::new(page_addr)).map_err(|_| {
                log::error!(
                    "munmap failed: could not unmap page at user heap position {:#x}",
                    page_addr
                );
                SyscallError::InvalidArgument
            })?;

            // NOTE: this might fuck me later
            if memory::pmm::is_usable_address(phys.as_u64()) {
                memory::pmm::free(phys.as_u64());
            } else {
                log::debug!(
                    "munmap: skipping free for non-usable physical page {:#x}",
                    phys.as_u64()
                );
            }
        }

        log::debug!("munmap: successfully unmapped pages");

        Ok(0)
    }
}

fn prot_to_flags(prot: u64) -> PageTableFlags {
    let mut flags = PageTableFlags::PRESENT;

    if prot != PROT_NONE {
        flags |= PageTableFlags::USER_ACCESSIBLE;
    }

    if prot & PROT_WRITE != 0 {
        flags |= PageTableFlags::WRITABLE;
    }

    if prot & PROT_EXEC == 0 {
        flags |= PageTableFlags::NO_EXECUTE;
    }

    flags
}

pub fn mprotect(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let addr = frame.rdi;
    let size = frame.rsi;
    let prot = frame.rdx;

    if size == 0 {
        return Err(SyscallError::InvalidArgument);
    }

    if (addr as usize & (PAGE_SIZE - 1)) != 0 {
        return Err(SyscallError::InvalidArgument);
    }

    let end = addr.checked_add(size).ok_or(SyscallError::InvalidArgument)?;
    let flags = prot_to_flags(prot);

    let current = system::proc::current()
        .ok_or(SyscallError::Other("no process found".into()))?;
    let mut proc = current.lock();

    let mut page = addr;
    while page < end {
        proc.address_space
            .as_mut()
            .unwrap()
            .update_page_flags(VirtAddr::new(page), flags)
            .map_err(|_| SyscallError::InvalidArgument)?;
        page += PAGE_SIZE as u64;
    }

    Ok(0)
}
