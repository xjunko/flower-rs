#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::fmt::Write;

use crate::sys::fs::{self};
extern crate alloc;

pub mod allocator;
pub mod auxv;
pub mod env;
pub mod file;
pub mod io;
pub mod process;
pub mod sys;
pub mod thread;

const MAX_PATH_BYTES: usize = 512;

pub fn with_c_path<T>(
    path: &[u8],
    f: impl FnOnce(*const u8) -> T,
) -> Option<T> {
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

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn with_c_path_raw<T>(
    path: *const u8,
    path_len: usize,
    f: impl FnOnce(*const u8) -> T,
) -> Option<T> {
    if path.is_null() || path_len >= MAX_PATH_BYTES {
        return None;
    }

    let slice = unsafe { core::slice::from_raw_parts(path, path_len) };

    if slice.last() == Some(&0) {
        return Some(f(path));
    }

    let mut path_buf = [0u8; MAX_PATH_BYTES];
    path_buf[..path_len].copy_from_slice(slice);
    path_buf[path_len] = 0;
    Some(f(path_buf.as_ptr()))
}

pub struct FlowerLibcStdout;
impl Write for FlowerLibcStdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        fs::write(1, s.as_ptr(), s.len());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut stdout = $crate::FlowerLibcStdout;
        let _ = core::write!(stdout, $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let err_str = b"application panicked!\n";
    fs::write(2, err_str.as_ptr(), err_str.len());
    let _ = Stderr.write_fmt(format_args!("panic info: {}\n", info));
    process::exit(1);
}

struct Stderr;

impl core::fmt::Write for Stderr {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        fs::write(2, s.as_ptr(), s.len());
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _init() {
    allocator::install();
    unsafe { auxv::init_current() };
}
