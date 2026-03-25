#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec;

use flower_libc::file::File;
use flower_libc::{env, print, println, process};

const PCM_BUFFER: usize = 4096;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let args: vec::Vec<&str> = env::args().collect();
    if args.len() < 2 {
        println!("usage: pcm <filename>");
        process::exit(1);
    }

    let file_path = args[1];

    println!("playing PCM file: {}", file_path);
    let ret_code = play(file_path);
    println!("pcm exited with code: {}", ret_code);

    process::exit(ret_code as u64);
}

pub fn play(args: &str) -> i32 {
    if args.is_empty() {
        println!("usage: pcm <filename>");
        return 1;
    }

    let music_file =
        File::open(args.to_string()).expect("failed to open music file");
    let driver_file = File::open("/dev/audio".to_string())
        .expect("failed to open driver file");

    let mut buffer = vec![0; PCM_BUFFER];
    let mut pcm_pos = 0;

    loop {
        let bytes_read = music_file.read(&mut buffer);
        if bytes_read.is_err() {
            break;
        }
        let bytes_read = bytes_read.unwrap();

        let mut total_written = 0;
        while total_written < bytes_read {
            let written =
                driver_file.write(&buffer[total_written..bytes_read]).unwrap();

            if written == 0 {
                println!("failed to write to audio driver");
                return 1;
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

    0
}
