#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use flower_libc::{env, println, process};

mod coreutils;
mod media;
mod other;

type StartFn = fn() -> Result<i32, Box<dyn core::error::Error>>;
// impl tables
const IMPL_TABLES: &[(&str, StartFn)] = &[
    // core-utils
    ("cat", coreutils::cat::start),
    ("echo", coreutils::echo::start),
    ("sleep", coreutils::sleep::start),
    // media-utils
    ("wav", media::wav::start),
    ("png", media::png::start),
    // other non-standard programs
    ("hello", other::hello::start),
    ("fetch", other::fetch::start),
];

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let mut args = env::args();
    let program_name =
        args.next().unwrap_or_default().split("/").last().unwrap_or_default();

    for (name, start_fn) in IMPL_TABLES {
        if *name == program_name {
            match start_fn() {
                Ok(exit_code) => process::exit(exit_code as u64),
                Err(e) => {
                    println!("Error: {}", e);
                    process::exit(1);
                },
            }
        }
    }
    0
}
