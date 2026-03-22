#![no_std]
#![no_main]

use flower_libc::{println, std};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe { flower_libc::auxv::init_current() };
    println!("hello from userspace rust!");
    std::exit(0);
}
