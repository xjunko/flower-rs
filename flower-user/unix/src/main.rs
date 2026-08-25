#![no_std]
#![no_main]

/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

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
