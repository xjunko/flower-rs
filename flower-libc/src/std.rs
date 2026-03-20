use flower_mono::syscalls::{
    SYS_CLOSE, SYS_EXIT, SYS_MSLEEP, SYS_OPEN, SYS_READ, SYS_WRITE,
};

use crate::syscalls::{syscall1, syscall3};

pub fn read(fd: u64, buf: &mut [u8]) -> usize {
    syscall3(SYS_READ, fd, buf.as_mut_ptr() as u64, buf.len() as u64) as usize
}

pub fn write(fd: u64, buf: &[u8]) -> usize {
    syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) as usize
}

pub fn open(path: &[u8], flags: u64) -> i64 {
    let result =
        syscall3(SYS_OPEN, path.as_ptr() as u64, path.len() as u64, flags);
    if result == u64::MAX { -1 } else { result as i64 }
}

pub fn close(fd: u64) -> i64 {
    let result = syscall1(SYS_CLOSE, fd);
    if result == u64::MAX { -1 } else { 0 }
}

pub fn exit(s: u64) -> ! {
    syscall1(SYS_EXIT, s);
    unreachable!();
}

pub fn sleep(millis: u64) { syscall1(SYS_MSLEEP, millis); }
