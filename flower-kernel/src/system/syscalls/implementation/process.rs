use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::{CStr, c_char};

use crate::system::syscalls::types::{SyscallError, SyscallFrame};
use crate::system::{self};

pub fn exit(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    system::proc::exit(frame.rdi);
    unreachable!();
}

pub fn msleep(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let ms = frame.rdi;
    system::proc::sleep(ms);
    Ok(0)
}

pub fn fork(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    system::proc::fork(frame).map_err(|e| {
        log::error!("fork failed: {}", e);
        SyscallError::Other
    })
}

pub fn waitpid(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    system::proc::waitpid(frame.rdi).map_err(|e| {
        if e == "no child process" {
            SyscallError::NoChildProcess
        } else {
            log::error!("waitpid failed: {}", e);
            SyscallError::Other
        }
    })
}

pub fn execve(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    const MAX_ARGV: usize = 32;

    let path_ptr = frame.rdi as *const c_char;
    if path_ptr.is_null() {
        return Err(SyscallError::InvalidArgument);
    }

    let path = unsafe { CStr::from_ptr(path_ptr) }
        .to_str()
        .map_err(|_| SyscallError::InvalidArgument)?;

    let argv_ptr = frame.rsi as *const *const c_char;
    let mut argv = Vec::<String>::new();

    if !argv_ptr.is_null() {
        for idx in 0..MAX_ARGV {
            let arg_ptr = unsafe { *argv_ptr.add(idx) };
            if arg_ptr.is_null() {
                break;
            }

            let arg = unsafe { CStr::from_ptr(arg_ptr) }
                .to_str()
                .map_err(|_| SyscallError::InvalidArgument)?;
            argv.push(arg.to_string());
        }
    }

    if argv.is_empty() {
        argv.push(path.to_string());
    }

    if let Err(reason) = system::proc::execve(path, &argv, frame) {
        log::error!("execve failed for path '{}': {:?}", path, reason);
        return Err(SyscallError::NoSuchFile);
    }
    Ok(0)
}
