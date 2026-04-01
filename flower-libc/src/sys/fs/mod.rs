use core::ptr;

use flower_mono::structs::FileStat;
use flower_mono::syscalls::{
    SYS_CLOSE, SYS_OPEN, SYS_READ, SYS_STAT, SYS_WRITE,
};

use crate::sys::kernel::{syscall_result, syscall1, syscall3};
use crate::with_c_path_raw;

#[unsafe(no_mangle)]
pub extern "C" fn read(fd: u64, buf: *mut u8, buf_len: usize) -> i64 {
    syscall3(SYS_READ, fd, buf as u64, buf_len as u64) as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn write(fd: u64, buf: *const u8, buf_len: usize) -> i64 {
    syscall3(SYS_WRITE, fd, buf as u64, buf_len as u64) as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn open(
    path: *const u8,
    path_len: usize,
    flags: u64,
    mode: u64,
) -> i64 {
    let result = match with_c_path_raw(path, path_len, |ptr| {
        syscall3(SYS_OPEN, ptr as u64, flags, mode)
    }) {
        Some(result) => result,
        None => return -1,
    };
    let result = syscall_result(result);
    if result < 0 { -1 } else { result }
}

#[unsafe(no_mangle)]
pub extern "C" fn close(fd: u64) -> i64 {
    let result = syscall_result(syscall1(SYS_CLOSE, fd));
    if result < 0 { -1 } else { 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn metadata(fd: u64, stat: *mut FileStat) -> i64 {
    if stat.is_null() {
        return -1;
    }
    unsafe {
        ptr::write(stat, FileStat::default());
    }
    let result = syscall3(SYS_STAT, fd, stat as u64, 0);
    if syscall_result(result) < 0 { -1 } else { 0 }
}
