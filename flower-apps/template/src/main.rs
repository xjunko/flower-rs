#![no_std]
#![no_main]

use flower_libc::std::exit;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe { flower_libc::auxv::init_current() };
    exit(0);
}
