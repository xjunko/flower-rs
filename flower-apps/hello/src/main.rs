#![no_std]
#![no_main]

use flower_libc::std;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    std::write(1, b"hello from userspace rust!\n");
    std::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { std::exit(1); }
