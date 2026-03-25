use flower_mono::syscalls::SYS_MSLEEP;

use crate::sys::kernel::syscall1;

pub fn sleep(millis: u64) { syscall1(SYS_MSLEEP, millis); }
