#![no_std]
#![no_main]

use flower_libc::std;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe { flower_libc::auxv::init_current() };
    std::write(1, b"hello from userspace rust!\n");
    std::exit(0);
}
