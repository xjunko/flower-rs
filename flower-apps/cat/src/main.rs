#![no_std]
#![no_main]

use flower_libc::{println, std};

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

    let file_fd = std::open(args.as_bytes(), 0, 0);
    if file_fd < 0 {
        println!("failed to open file");
        return 1;
    }

    let mut buffer = [0u8; 1024];
    loop {
        let read_bytes = std::read(file_fd as u64, &mut buffer);
        if read_bytes <= 0 {
            break;
        }
        std::write(1, &buffer[..read_bytes as usize]);
    }
    std::close(file_fd as u64);

    0
}
