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

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use flower_libc::file::File;
use flower_libc::sys::fs::bits::FS_RDONLY;
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

    if let Ok(file) = File::open(args.to_string(), FS_RDONLY) {
        let mut buffer = [0u8; 1024];
        loop {
            let read_bytes = file.read(&mut buffer).unwrap_or(0);
            if read_bytes == 0 {
                break;
            }
            print!(
                "{}",
                core::str::from_utf8(&buffer[..read_bytes]).unwrap_or("<invalid utf-8>")
            );
        }
    } else {
        println!("failed to open file: {}", args);
        return 1;
    }

    0
}
