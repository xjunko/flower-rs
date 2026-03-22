#![no_std]
pub mod auxv;
pub mod std;
pub mod syscalls;
pub mod tty;
pub mod utils;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! { std::panic(info) }

pub fn _init() { unsafe { auxv::init_current() }; }
