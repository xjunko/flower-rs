use flower_mono::syscalls::SYS_MTIME;

use crate::sys::kernel::syscall1;

pub struct Duration {
    millis: u64,
}

impl Duration {
    pub fn as_millis(&self) -> u64 { self.millis }

    pub fn as_secs(&self) -> u64 { self.millis / 1000 }
}

pub struct SystemTime {
    millis: u64,
}

impl SystemTime {
    pub fn now() -> Self { SystemTime { millis: __sys_get_time_ms() } }

    pub fn elapsed(&self) -> Duration {
        Duration { millis: self.elapsed_millis() }
    }

    pub fn elapsed_millis(&self) -> u64 { __sys_get_time_ms() - self.millis }

    pub fn as_millis(&self) -> u64 { self.millis }
}

fn __sys_get_time_ms() -> u64 {
    let mut time: u64 = 0;
    syscall1(SYS_MTIME, &mut time as *mut u64 as u64);
    time
}
