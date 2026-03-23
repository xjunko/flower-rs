#![no_std]
#![no_main]

use flower_libc::{println, std};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let argc = flower_libc::auxv::argc();
    println!("hello from userspace rust!");
    println!("argc = {}", argc);
    for i in 0..argc {
        if let Some(arg) = flower_libc::auxv::argv(i) {
            println!("argv[{}] = {}", i, arg);
        }
    }

    std::exit(0);
}
