use alloc::boxed::Box;
use alloc::vec::Vec;

use flower_libc::thread::sleep;
use flower_libc::{env, println};

pub fn start() -> Result<i32, Box<dyn core::error::Error>> {
    let args: Vec<&str> = env::args().collect();
    if args.len() < 2 {
        println!("missing argument: number of milliseconds to sleep");
        return Ok(1);
    }
    let millis: u64 = args[1].parse()?;
    sleep(millis);
    Ok(0)
}
