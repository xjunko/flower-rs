use alloc::boxed::Box;

use flower_libc::{env, println};

pub fn start() -> Result<i32, Box<dyn core::error::Error>> {
    for arg in env::args().skip(1) {
        println!("Hello, {}!", arg);
    }
    Ok(0)
}
