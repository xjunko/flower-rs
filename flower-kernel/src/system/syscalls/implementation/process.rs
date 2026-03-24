use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::{CStr, c_char};

use x86_64::VirtAddr;
use x86_64::registers::model_specific::FsBase;
use x86_64::structures::paging::PageTableFlags;

use crate::system::mem::PAGE_SIZE;
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

pub fn fork(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    system::proc::fork(frame).map_err(|e| {
        log::error!("fork failed: {}", e);
        SyscallError::Other
    })
}

pub fn execve(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    const MAX_ARGV: usize = 32;

    let path_ptr = frame.rdi as *const c_char;
    if path_ptr.is_null() {
        return Err(SyscallError::InvalidArgument);
    }

    let path = unsafe { CStr::from_ptr(path_ptr) }
        .to_str()
        .map_err(|_| SyscallError::InvalidArgument)?;

    let argv_ptr = frame.rsi as *const *const c_char;
    let mut argv = Vec::<String>::new();

    if !argv_ptr.is_null() {
        for idx in 0..MAX_ARGV {
            let arg_ptr = unsafe { *argv_ptr.add(idx) };
            if arg_ptr.is_null() {
                break;
            }

            let arg = unsafe { CStr::from_ptr(arg_ptr) }
                .to_str()
                .map_err(|_| SyscallError::InvalidArgument)?;
            argv.push(arg.to_string());
        }
    }

    if argv.is_empty() {
        argv.push(path.to_string());
    }

    if let Err(reason) = system::proc::execve(path, &argv, frame) {
        log::error!("execve failed for path '{}': {:?}", path, reason);
        return Err(SyscallError::NoSuchFile);
    }
    Ok(0)
}

pub fn write_fsbase(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let arg1 = frame.rdi;
    if let fsbase = VirtAddr::new(arg1)
        && system::mem::vmm::page_is_mapped(fsbase)
    {
        log::debug!("writing fsbase with value {:#x}", arg1);
        let fsbase = arg1;
        FsBase::write(VirtAddr::new(fsbase));
        Ok(0)
    } else {
        Err(SyscallError::InvalidArgument)
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

    if fd != -1 {
        return Err(SyscallError::InvalidArgument);
    }

    {
        let current = system::proc::current().ok_or(SyscallError::Other)?;
        let mut proc = current.lock();

        let heap_start = proc.user_heap_position;
        let heap_pages = (size + system::mem::PAGE_SIZE as u64 - 1)
            / system::mem::PAGE_SIZE as u64;

        let mut heap_ptr = heap_start;

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
            heap_ptr += system::mem::PAGE_SIZE as u64;
        }

        proc.user_heap_position = heap_ptr;
        Ok(heap_start)
    }
}

pub fn munmap(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let addr = frame.rdi;
    let base = addr & !(PAGE_SIZE as u64 - 1);
    let size = frame.rsi;

    if size == 0 {
        return Err(SyscallError::InvalidArgument);
    }

    let end = addr.checked_add(size).ok_or(SyscallError::InvalidArgument)?;

    let pages = (size + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;

    log::debug!("munmap: addr={:#x}, size={}, pages={}", addr, size, pages);

    {
        let current = system::proc::current().ok_or(SyscallError::Other)?;
        let mut proc = current.lock();

        if addr < proc.user_heap || end > proc.user_heap_position {
            return Err(SyscallError::InvalidArgument);
        }

        for i in 0..pages {
            let page_addr = base + i * PAGE_SIZE as u64;
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
