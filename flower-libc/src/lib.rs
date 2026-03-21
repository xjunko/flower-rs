#![no_std]
pub mod std;
pub mod syscalls;
pub mod utils;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! { std::panic(info) }
