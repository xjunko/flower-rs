use flower_mono::syscalls::{SYS_MMAP, SYS_MUNMAP};
pub use flower_mono::syscalls::{
    MAP_ANONYMOUS, MAP_FIXED, MAP_PRIVATE, MAP_SHARED, PROT_EXEC, PROT_NONE,
    PROT_READ, PROT_WRITE,
};

use crate::sys::kernel::{syscall_result, syscall2, syscall6};

pub fn mmap(
    addr: *mut u8,
    size: usize,
    prot: u64,
    flags: u64,
    fd: u64,
    offset: usize,
) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    let ret = syscall6(
        SYS_MMAP,
        addr as u64,
        size as u64,
        prot,
        flags,
        fd,
        offset as u64,
    );
    let ret = syscall_result(ret);

    if ret < 0 { core::ptr::null_mut() } else { ret as *mut u8 }
}

pub fn mmap_anonymous(size: usize) -> *mut u8 {
    mmap(
        core::ptr::null_mut(),
        size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        u64::MAX,
        0,
    )
}

pub fn munmap(addr: *mut u8, size: usize) -> i64 {
    if addr.is_null() || size == 0 {
        return -1;
    }

    let ret = syscall_result(syscall2(SYS_MUNMAP, addr as u64, size as u64));
    if ret < 0 { -1 } else { 0 }
}
