use crate::system::syscalls::types::SyscallError;
use crate::system::{self};

pub fn exit(
    _arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> Result<u64, SyscallError> {
    system::proc::exit();
    unreachable!();
}

pub fn msleep(
    arg1: u64,
    _arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> Result<u64, SyscallError> {
    let ms = arg1;
    system::proc::sleep(ms);
    Ok(0)
}
