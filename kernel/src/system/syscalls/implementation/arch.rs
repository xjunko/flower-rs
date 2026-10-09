use defs::prctl::ARCH_SET_FS;
use x86_64::VirtAddr;
use x86_64::registers::model_specific::FsBase;

use crate::system::mem::vmm::AddressSpace;
use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallError;

pub fn ctl(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    // TODO: should this be kernel or userspace?
    let current_space = AddressSpace::current();

    let arg1 = frame.rdi;
    let arg2 = frame.rsi;

    match arg1 {
        ARCH_SET_FS => {
            if let fs_base = VirtAddr::new(arg2)
                && current_space.is_mapped(fs_base)
            {
                log::debug!("writing FSBase with: {:#x}", fs_base);
                FsBase::write(fs_base);
                Ok(0)
            } else {
                Err(SyscallError::InvalidArgument)
            }
        },
        _ => Err(SyscallError::NoPermission),
    }
}
