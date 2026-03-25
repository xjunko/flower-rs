#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;

use flower_libc::{env, print, println, std};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let args: Vec<&str> = env::args().collect();
    if args.len() < 2 {
        println!("");
        std::exit(0);
    }

    for word in args.into_iter().skip(1) {
        print!("{} ", word);
    }
    println!("");

    std::exit(0);
}
