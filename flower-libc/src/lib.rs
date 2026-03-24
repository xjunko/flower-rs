#![no_std]

use core::fmt::Write;
extern crate alloc;

pub mod allocator;
pub mod auxv;
pub mod std;
pub mod syscalls;
pub mod tty;

pub struct FlowerLibcStdout;
impl Write for FlowerLibcStdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        std::write(1, s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut stdout = $crate::FlowerLibcStdout;
        let _ = core::write!(stdout, $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! { std::panic(info) }

pub fn _init() {
    allocator::install();
    unsafe { auxv::init_current() };
}
