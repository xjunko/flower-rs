use flower_mono::structs::FileStat;
use flower_mono::syscalls::{
    SYS_CLOSE, SYS_OPEN, SYS_READ, SYS_STAT, SYS_WRITE,
};

use crate::sys::kernel::{syscall_result, syscall1, syscall3};
use crate::with_c_path;

pub fn read(fd: u64, buf: &mut [u8]) -> i64 {
    syscall3(SYS_READ, fd, buf.as_mut_ptr() as u64, buf.len() as u64) as i64
}

pub fn write(fd: u64, buf: &[u8]) -> i64 {
    syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) as i64
}

pub fn open(path: &[u8], flags: u64, mode: u64) -> i64 {
    let result = match with_c_path(path, |ptr| {
        syscall3(SYS_OPEN, ptr as u64, flags, mode)
    }) {
        Some(result) => result,
        None => return -1,
    };
    let result = syscall_result(result);

    if result < 0 { -1 } else { result }
}

pub fn close(fd: u64) -> i64 {
    let result = syscall_result(syscall1(SYS_CLOSE, fd));
    if result < 0 { -1 } else { 0 }
}

pub fn metadata(fd: u64) -> Option<FileStat> {
    let metadata: FileStat = FileStat { size: 0 };
    let result = syscall3(SYS_STAT, fd, &metadata as *const FileStat as u64, 0);
    if syscall_result(result) < 0 { None } else { Some(metadata) }
}
