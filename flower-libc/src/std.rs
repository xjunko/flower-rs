use crate::syscalls::{syscall1, syscall3};

pub const SYS_EXIT: u64 = 0;
pub const SYS_WRITE: u64 = 1;

pub fn exit(s: u64) -> ! {
    syscall1(SYS_EXIT, s);
    unreachable!();
}

pub fn write(fd: u64, buf: &[u8]) -> usize {
    syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) as usize
}
