use flower_mono::syscalls::{SYS_MMAP, SYS_MUNMAP};

use crate::sys::kernel::{syscall_result, syscall2, syscall6};

pub fn mmap(fd: u64, size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    let ret = syscall6(SYS_MMAP, 0, size as u64, 0, 0, fd, 0);
    let ret = syscall_result(ret);

    if ret < 0 { core::ptr::null_mut() } else { ret as *mut u8 }
}

pub fn munmap(addr: *mut u8, size: usize) -> i64 {
    if addr.is_null() || size == 0 {
        return -1;
    }

    let ret = syscall_result(syscall2(SYS_MUNMAP, addr as u64, size as u64));
    if ret < 0 { -1 } else { 0 }
}
