#![no_std]
#![no_main]

use flower_libc::std::exit;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { exit(0); }
