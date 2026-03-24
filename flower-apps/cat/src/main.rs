#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use flower_libc::file::File;
use flower_libc::{print, println, std};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let argc = flower_libc::auxv::argc();

    if argc < 2 {
        println!("usage: cat <filename>");
        std::exit(0);
    }

    let file_path = match flower_libc::auxv::argv(1) {
        Some(path) => path,
        None => {
            println!("failed to get filename argument");
            std::exit(1);
        },
    };

    std::exit(cat(file_path) as u64);
}

pub fn cat(args: &str) -> i32 {
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
        file.close().ok();
    } else {
        println!("failed to open file");
        return 1;
    }

    0
}
