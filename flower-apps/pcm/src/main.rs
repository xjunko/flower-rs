#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;

use flower_libc::{print, println, std};

const PCM_BUFFER: usize = 4096;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let argc = flower_libc::auxv::argc();
    if argc < 2 {
        println!("usage: pcm <filename>");
        std::exit(1);
    }

    let file_path = match flower_libc::auxv::argv(1) {
        Some(path) => path,
        None => {
            println!("failed to get filename argument");
            std::exit(1);
        },
    };

    println!("playing PCM file: {}", file_path);
    let ret_code = play(file_path);
    println!("pcm exited with code: {}", ret_code);

    std::exit(ret_code as u64);
}

pub fn play(args: &str) -> i32 {
    if args.is_empty() {
        println!("usage: pcm <filename>");
        return -1;
    }

    let file_fd = std::open(args.as_bytes(), 0, 0);
    if file_fd < 0 {
        println!("failed to open file");
        return -1;
    }

    let driver_fd = std::open(b"/dev/audio\0", 0, 0);
    if driver_fd < 0 {
        println!("failed to open audio driver");
        return -1;
    }

    let mut buffer = vec![0; PCM_BUFFER];

    let mut pcm_pos = 0;

    loop {
        let bytes_read = std::read(file_fd as u64, &mut buffer);
        if bytes_read <= 0 {
            break;
        }

        let mut total_written = 0;
        while total_written < bytes_read {
            let written = std::write(
                driver_fd as u64,
                &buffer[total_written as usize..bytes_read as usize],
            );
            if written < 0 {
                println!("failed to write to audio driver");
                std::close(driver_fd as u64);
                std::close(file_fd as u64);
                return -1;
            }

            total_written += written;
            print!(
                "\rplayed {} bytes ({} total)",
                written,
                pcm_pos + total_written,
            );
        }
        pcm_pos += bytes_read;
    }
    std::close(driver_fd as u64);
    std::close(file_fd as u64);

    0
}
