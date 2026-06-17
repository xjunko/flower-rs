use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use flower_libc::file::File;
use flower_libc::{env, print, println};

extern crate alloc;

pub fn start() -> Result<i32, Box<dyn core::error::Error>> {
    let args: Vec<&str> = env::args().collect();
    if args.len() < 2 {
        println!("usage: cat <filename>");
        return Ok(0);
    }

    let file_path = args[1];

    Ok(cat(file_path))
}

fn cat(args: &str) -> i32 {
    if args.is_empty() {
        println!("usage: cat <filename>");
        return 1;
    }

    if let Ok(file) = File::open(args.to_string()) {
        let mut buffer = [0u8; 1024];
        loop {
            let read_bytes = file.read(&mut buffer).unwrap_or(0);
            if read_bytes == 0 {
                break;
            }
            print!(
                "{}",
                core::str::from_utf8(&buffer[..read_bytes])
                    .unwrap_or("<invalid utf-8>")
            );
        }
    } else {
        println!("failed to open file: {}", args);
        return 1;
    }

    0
}
