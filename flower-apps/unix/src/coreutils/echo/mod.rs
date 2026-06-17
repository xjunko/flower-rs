use alloc::boxed::Box;
use alloc::vec::Vec;

use flower_libc::{env, print, println};

pub fn start() -> Result<i32, Box<dyn core::error::Error>> {
    let args: Vec<&str> = env::args().collect();
    if args.len() < 2 {
        println!("");
        return Ok(1);
    }

    for word in args.into_iter().skip(1) {
        print!("{} ", word);
    }
    println!("");

    Ok(0)
}
