use core::fmt::Write;
use core::panic::PanicInfo;

use flower_mono::syscalls::{
    SYS_CLOSE, SYS_EXECVE, SYS_EXIT, SYS_FORK, SYS_MSLEEP, SYS_OPEN, SYS_READ,
    SYS_WRITE,
};

use crate::syscalls::{syscall0, syscall1, syscall3};
use crate::utils::CStr;

pub fn read(fd: u64, buf: &mut [u8]) -> i64 {
    syscall3(SYS_READ, fd, buf.as_mut_ptr() as u64, buf.len() as u64) as i64
}

pub fn write(fd: u64, buf: &[u8]) -> i64 {
    syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) as i64
}

pub fn open(path: &[u8], flags: u64, mode: u64) -> i64 {
    let c_path = CStr::from_bytes_with_nul(path)
        .expect("path did not contain null byte");
    let result = syscall3(SYS_OPEN, c_path.as_ptr() as u64, flags, mode);
    if result == u64::MAX { -1 } else { result as i64 }
}

pub fn close(fd: u64) -> i64 {
    let result = syscall1(SYS_CLOSE, fd);
    if result == u64::MAX { -1 } else { 0 }
}

pub fn fork() -> i64 { syscall0(SYS_FORK) as i64 }

pub fn execve(path: &[u8], argv: u64, envp: u64) -> i64 {
    let c_path = CStr::from_bytes_with_nul(path)
        .expect("path did not contain null byte");
    syscall3(SYS_EXECVE, c_path.as_ptr() as u64, argv, envp) as i64
}

pub fn exit(s: u64) -> ! {
    syscall1(SYS_EXIT, s);
    unreachable!();
}

pub fn sleep(millis: u64) { syscall1(SYS_MSLEEP, millis); }

struct Stderr;

impl core::fmt::Write for Stderr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(2, s.as_bytes());
        Ok(())
    }
}

pub fn panic(info: &PanicInfo) -> ! {
    write(2, b"application panicked!\n");
    let _ = Stderr.write_fmt(format_args!("panic info: {}\n", info));
    exit(1);
}
