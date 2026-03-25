#![no_std]
#![no_main]

extern crate alloc;

mod resample;
mod wav;

use alloc::string::ToString;
use alloc::vec;

use flower_libc::file::File;
use flower_libc::{env, println, process};

use crate::resample::resample_linear_bits;

const DRIVER_BUFFER: usize = 4096;

const TARGET_SAMPLE_RATE: usize = 48000;
const TARGET_CHANNELS: usize = 2;
const TARGET_BITS_PER_SAMPLE: usize = 16;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let args: vec::Vec<&str> = env::args().collect();
    if args.len() < 2 {
        println!("usage: wav <filename>");
        process::exit(1);
    }

    let file_path = args[1];

    println!("playing WAV file: {}", file_path);
    let ret_code = play(file_path);
    println!("wav exited with code: {}", ret_code);

    process::exit(ret_code as u64);
}

pub fn play(args: &str) -> i32 {
    if args.is_empty() {
        println!("usage: wav <filename>");
        return 1;
    }

    let music_file =
        File::open(args.to_string()).expect("failed to open music file");
    let driver_file = File::open("/dev/audio".to_string())
        .expect("failed to open driver file");

    let music_metadata =
        music_file.metadata().expect("failed to get music file metadata");

    let mut music_buffer = vec![0u8; music_metadata.size];
    music_file.read(&mut music_buffer).expect("failed to read music file");

    let wav =
        wav::parse_wav(&music_buffer).expect("failed to decode wav music");

    println!(
        "playing audio w/ {} Hz, {} channels, {} bits",
        wav.sample_rate, wav.channels, wav.bits_per_sample
    );

    let target_buffer_size =
        DRIVER_BUFFER * (TARGET_SAMPLE_RATE / wav.sample_rate as usize);

    if wav.sample_rate as usize != TARGET_SAMPLE_RATE {
        println!(
            "resampling to {} Hz, {} channels, {} bits w/ buffer size {}",
            TARGET_SAMPLE_RATE,
            TARGET_CHANNELS,
            TARGET_BITS_PER_SAMPLE,
            target_buffer_size
        );
    }

    let mut total_bytes = 0;
    let mut out_samples: vec::Vec<i16> = vec![(DRIVER_BUFFER * 3) as i16];

    while total_bytes < wav.data.len() {
        let end = (total_bytes + DRIVER_BUFFER).min(wav.data.len());
        let chunk = &wav.data[total_bytes..end];

        out_samples.clear();
        resample_linear_bits(
            chunk,
            wav.bits_per_sample,
            wav.sample_rate,
            TARGET_SAMPLE_RATE as u32,
            &mut out_samples,
        );

        let out_bytes: &[u8] = unsafe {
            core::slice::from_raw_parts(
                out_samples.as_ptr() as *const u8,
                out_samples.len() * 2,
            )
        };

        let mut written_total = 0;
        while written_total < out_bytes.len() {
            let written =
                driver_file.write(&out_bytes[written_total..]).unwrap();
            if written == 0 {
                println!("failed to write to audio driver");
                return 1;
            }
            written_total += written;
        }

        total_bytes += chunk.len();
    }

    0
}
