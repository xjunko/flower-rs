use flower_mono::syscalls::{SYS_EXECVE, SYS_EXIT, SYS_FORK, SYS_WAITPID};

use crate::sys::kernel::{syscall_result, syscall0, syscall1, syscall3};
use crate::{allocator, with_c_path_raw};

#[unsafe(no_mangle)]
pub extern "C" fn fork() -> i64 { syscall0(SYS_FORK) as i64 }

#[unsafe(no_mangle)]
pub extern "C" fn waitpid(pid: u64) -> i64 {
    let result = syscall_result(syscall1(SYS_WAITPID, pid));
    if result < 0 { -1 } else { result }
}

#[unsafe(no_mangle)]
pub extern "C" fn execve(
    path: *const u8,
    path_len: usize,
    argv: u64,
    envp: u64,
) -> i64 {
    match with_c_path_raw(path, path_len, |ptr| {
        syscall3(SYS_EXECVE, ptr as u64, argv, envp)
    }) {
        Some(result) => {
            let result = syscall_result(result);
            if result < 0 { -1 } else { result }
        },
        None => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn exit(s: u64) -> ! {
    allocator::uninstall();
    syscall1(SYS_EXIT, s);
    unreachable!();
}
