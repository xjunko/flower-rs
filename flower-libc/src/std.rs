use flower_mono::syscalls::{SYS_EXIT, SYS_MSLEEP, SYS_WRITE};

use crate::syscalls::{syscall1, syscall3};

pub fn exit(s: u64) -> ! {
    syscall1(SYS_EXIT, s);
    unreachable!();
}

pub fn write(fd: u64, buf: &[u8]) -> usize {
    syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) as usize
}

pub fn sleep(millis: u64) { syscall1(SYS_MSLEEP, millis); }
