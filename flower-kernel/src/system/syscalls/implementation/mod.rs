use flower_mono::syscalls::{
    SYS_CLOSE, SYS_EXECVE, SYS_EXIT, SYS_FORK, SYS_MMAP, SYS_MSLEEP,
    SYS_MUNMAP, SYS_OPEN, SYS_READ, SYS_SEEK, SYS_WAITPID, SYS_WRITE,
    SYS_WRITE_FS_BASE,
};

mod fs;
mod process;

use crate::system::syscalls::types::SyscallHandler;

pub static SYSCALL_HANDLERS: [Option<SyscallHandler>; 256] = {
    let mut handlers = [None; 256];

    handlers[SYS_EXIT as usize] = Some(process::exit as SyscallHandler);
    handlers[SYS_FORK as usize] = Some(process::fork as SyscallHandler);
    handlers[SYS_WAITPID as usize] = Some(process::waitpid as SyscallHandler);
    handlers[SYS_EXECVE as usize] = Some(process::execve as SyscallHandler);

    handlers[SYS_READ as usize] = Some(fs::read as SyscallHandler);
    handlers[SYS_WRITE as usize] = Some(fs::write as SyscallHandler);
    handlers[SYS_OPEN as usize] = Some(fs::open as SyscallHandler);
    handlers[SYS_CLOSE as usize] = Some(fs::close as SyscallHandler);
    handlers[SYS_SEEK as usize] = Some(fs::seek as SyscallHandler);

    handlers[SYS_MSLEEP as usize] = Some(process::msleep as SyscallHandler);

    handlers[SYS_WRITE_FS_BASE as usize] =
        Some(process::write_fsbase as SyscallHandler);

    handlers[SYS_MMAP as usize] = Some(process::mmap as SyscallHandler);
    handlers[SYS_MUNMAP as usize] = Some(process::munmap as SyscallHandler);

    handlers
};
