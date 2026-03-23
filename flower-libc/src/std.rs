use core::fmt::Write;
use core::panic::PanicInfo;

use flower_mono::syscalls::{
    SYS_CLOSE, SYS_EXECVE, SYS_EXIT, SYS_FORK, SYS_MMAP, SYS_MSLEEP, SYS_OPEN,
    SYS_READ, SYS_WRITE,
};

use crate::syscalls::{syscall0, syscall1, syscall3, syscall6};

const MAX_PATH_BYTES: usize = 512;

fn with_c_path<T>(path: &[u8], f: impl FnOnce(*const u8) -> T) -> Option<T> {
    if path.last() == Some(&0) {
        return Some(f(path.as_ptr()));
    }

    if path.len() + 1 > MAX_PATH_BYTES {
        return None;
    }

    let mut path_buf = [0u8; MAX_PATH_BYTES];
    path_buf[..path.len()].copy_from_slice(path);
    path_buf[path.len()] = 0;
    Some(f(path_buf.as_ptr()))
}

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
    if result == u64::MAX { -1 } else { result as i64 }
}

pub fn close(fd: u64) -> i64 {
    let result = syscall1(SYS_CLOSE, fd);
    if result == u64::MAX { -1 } else { 0 }
}

pub fn fork() -> i64 { syscall0(SYS_FORK) as i64 }

pub fn execve(path: &[u8], argv: u64, envp: u64) -> i64 {
    match with_c_path(path, |ptr| syscall3(SYS_EXECVE, ptr as u64, argv, envp))
    {
        Some(result) => result as i64,
        None => -1,
    }
}

pub fn exit(s: u64) -> ! {
    syscall1(SYS_EXIT, s);
    unreachable!();
}

pub fn sleep(millis: u64) { syscall1(SYS_MSLEEP, millis); }

pub fn mmap(size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    let ret = syscall6(SYS_MMAP, 0, size as u64, 0, 0, u64::MAX, 0);

    if ret == u64::MAX { core::ptr::null_mut() } else { ret as *mut u8 }
}

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
