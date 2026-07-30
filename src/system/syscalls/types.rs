use alloc::string::String;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: isize,

    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

#[derive(Debug)]
pub enum SyscallError {
    NotPermitted,
    NoSuchFile,
    NoProcess,
    InterruptedSyscall,
    IOError,
    NoDeviceOrAddress,
    NoExecutable,
    BadFileDescriptor,
    NoChildProcess,
    ResourceTryAgain,
    NoMemory,
    NoPermission,
    BadAddress,
    BlockDeviceRequired,
    ResourceBusy,
    InvalidArgument,
    NoSpace,
    TooLong,
    Other(String),
}

impl SyscallError {
    pub fn errno(&self) -> i64 {
        match self {
            SyscallError::NotPermitted => 1,         // EPERM
            SyscallError::NoSuchFile => 2,           // ENOENT
            SyscallError::NoProcess => 3,            // ESRCH
            SyscallError::InterruptedSyscall => 4,   // EINTR
            SyscallError::IOError => 5,              // EIO
            SyscallError::NoDeviceOrAddress => 6,    // ENXIO
            SyscallError::NoExecutable => 8,         // ENOEXEC
            SyscallError::BadFileDescriptor => 9,    // EBADF
            SyscallError::NoChildProcess => 10,      // ECHILD
            SyscallError::ResourceTryAgain => 11,    // EAGAIN
            SyscallError::NoMemory => 12,            // ENOMEM
            SyscallError::NoPermission => 13,        // EACCES
            SyscallError::BadAddress => 14,          // EFAULT
            SyscallError::BlockDeviceRequired => 15, // ENOTBLK
            SyscallError::ResourceBusy => 16,        // EBUSY
            SyscallError::InvalidArgument => 22,     // EINVAL
            SyscallError::NoSpace => 28,             // ENOSPC
            SyscallError::TooLong => 36,             // ENAMETOOLONG
            SyscallError::Other(_) => 255,
        }
    }
}

pub type SyscallHandler = fn(&mut SyscallFrame) -> Result<u64, SyscallError>;
