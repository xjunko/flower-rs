use flower_mono::syscalls::{
    SYS_CLOSE, SYS_EXIT, SYS_MSLEEP, SYS_OPEN, SYS_READ, SYS_WRITE,
};

mod fs;
mod process;

use crate::system::syscalls::types::SyscallHandler;

pub static SYSCALL_HANDLERS: [Option<SyscallHandler>; 256] = {
    let mut handlers = [None; 256];

    handlers[SYS_EXIT as usize] = Some(process::exit as SyscallHandler);

    handlers[SYS_READ as usize] = Some(fs::read as SyscallHandler);
    handlers[SYS_WRITE as usize] = Some(fs::write as SyscallHandler);
    handlers[SYS_OPEN as usize] = Some(fs::open as SyscallHandler);
    handlers[SYS_CLOSE as usize] = Some(fs::close as SyscallHandler);

    handlers[SYS_MSLEEP as usize] = Some(process::msleep as SyscallHandler);

    handlers
};
