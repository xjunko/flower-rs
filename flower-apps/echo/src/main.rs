#![no_std]
#![no_main]

use flower_libc::{print, println, std};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let argc = flower_libc::auxv::argc();
    if argc < 2 {
        println!("");
        std::exit(0);
    }

    for i in 1..argc {
        let arg = flower_libc::auxv::argv(i).unwrap_or_default();
        print!("{} ", arg);
    }
    println!("");
    std::exit(0);
}
