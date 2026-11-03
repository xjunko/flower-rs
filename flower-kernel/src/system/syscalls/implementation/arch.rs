use flower_mono::prctl::ARCH_SET_FS;
use x86_64::VirtAddr as x86VirtAddr;
use x86_64::registers::model_specific::FsBase;

use crate::arch::addr::VirtAddr;
use crate::system::mem::vmm::AddressSpace;
use crate::system::syscalls::SyscallFrame;
use crate::system::syscalls::types::SyscallError;

pub fn ctl(frame: &mut SyscallFrame) -> Result<u64, SyscallError> {
    let address_space = AddressSpace::current();

    let arg1 = frame.rdi;
    let arg2 = frame.rsi;

    match arg1 {
        ARCH_SET_FS => {
            if let fs_base = VirtAddr::new(arg2)
                && address_space.is_mapped(fs_base)
            {
                log::debug!("writing FSBase with: {:#x}", fs_base);
                FsBase::write(x86VirtAddr::new(fs_base.as_u64()));
                Ok(0)
            } else {
                Err(SyscallError::InvalidArgument)
            }
        },
        _ => Err(SyscallError::NoPermission),
    }
}
