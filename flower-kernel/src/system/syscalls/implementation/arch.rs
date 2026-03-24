use x86_64::VirtAddr;
use x86_64::registers::model_specific::FsBase;

use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallError;
use crate::system::{self};

pub fn write_fsbase(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let arg1 = frame.rdi;
    if let fsbase = VirtAddr::new(arg1)
        && system::mem::vmm::page_is_mapped(fsbase)
    {
        log::debug!("writing fsbase with value {:#x}", arg1);
        let fsbase = arg1;
        FsBase::write(VirtAddr::new(fsbase));
        Ok(0)
    } else {
        Err(SyscallError::InvalidArgument)
    }
}
