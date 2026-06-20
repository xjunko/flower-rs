use crate::system::vfs::VFSError;

pub mod elf;
pub mod mem;
pub mod proc;
pub mod vfs;

pub enum KernelError {
    FileSystem(VFSError),
}

pub type KernelResult<T> = Result<T, KernelError>;
