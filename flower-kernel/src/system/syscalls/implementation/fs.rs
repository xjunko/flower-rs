use crate::system::syscalls::types::SyscallError;
use crate::system::vfs::{FdKind, VFSError};
use crate::system::{self};
use crate::{error, print};

pub fn read(
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> Result<u64, SyscallError> {
    let fd = arg1 as usize;
    let buf = arg2 as *mut u8;
    let len = arg3 as usize;

    let result =
        system::proc::with_fd_table(|table| match table.get_mut(fd)? {
            FdKind::File(file) => {
                let slice =
                    unsafe { core::slice::from_raw_parts_mut(buf, len) };
                file.read(slice)
            },
            _ => {
                error!("read syscall: fd {} is not readable", fd);
                Err(VFSError::PermissionDenied)
            },
        });

    Ok(result.unwrap_or(0) as u64)
}

pub fn write(
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> Result<u64, SyscallError> {
    let fd = arg1 as usize;
    let buf = arg2 as *const u8;
    let len = arg3 as usize;

    let result = system::proc::with_fd_table(|table| match table.get(fd)? {
        FdKind::Stdout | FdKind::Stderr => {
            for i in 0..len {
                let byte = unsafe { *buf.add(i) };
                print!("{}", byte as char);
            }
            Ok(len)
        },
        _ => {
            error!("write syscall: fd {} is not writable", fd);
            Err(VFSError::PermissionDenied)
        },
    });

    Ok(result.unwrap_or(0) as u64)
}

pub fn open(
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> Result<u64, SyscallError> {
    let path_buf = arg1 as *const u8;
    let path_len = arg2 as usize;
    let flags = arg3 as u32;

    let path = unsafe {
        let slice = core::slice::from_raw_parts(path_buf, path_len);
        core::str::from_utf8_unchecked(slice)
    };

    // TODO: handle directory
    match system::vfs::open(path, flags) {
        Ok(file) => {
            let result = system::proc::with_fd_table(|table| {
                table.alloc(FdKind::File(file))
            });
            Ok(result.map(|fd| fd as u64).unwrap_or(u64::MAX))
        },
        Err(_) => Err(SyscallError::NotFound),
    }
}

pub fn close(
    arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> Result<u64, SyscallError> {
    let fd = arg1 as usize;
    let result = system::proc::with_fd_table(|table| table.close(fd));
    if result.is_ok() { Ok(0) } else { Err(SyscallError::NotFound) }
}
