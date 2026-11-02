use crate::system::syscalls::SyscallError;
use crate::system::vfs::VFSError;

pub mod elf;
pub mod mem;
pub mod proc;
pub mod syscalls;
pub mod vfs;

pub enum KernelError {
    FileSystem(VFSError),
}

pub type KernelResult<T> = Result<T, KernelError>;

pub trait ToSyscallError {
    fn to_syscall_error(&self) -> SyscallError;
}

impl ToSyscallError for KernelError {
    fn to_syscall_error(&self) -> SyscallError {
        match self {
            Self::FileSystem(err) => err.to_syscall_error(),
        }
    }
}
