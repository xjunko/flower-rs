#![no_std]
#![no_main]

use flower_libc::{env, println, process};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    for arg in env::args().skip(1) {
        println!("Hello, {}!", arg);
    }

    process::exit(0);
}
