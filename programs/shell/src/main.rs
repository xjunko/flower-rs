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

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use flower_libc::file::File;
use flower_libc::{io, print, println, process};

mod tools;

const BUFFER_SIZE: usize = 64;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    tools::exec::run_quiet("/init/bin/fetch");

    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        print!(">");

        let len = io::read_line(&mut buf);
        if len == 0 {
            continue;
        }
        buf[len..BUFFER_SIZE].fill(0);

        let buf_nulled =
            buf.iter().copied().take_while(|&b| b != 0).collect::<Vec<u8>>();
        let input = String::from_utf8(buf_nulled).unwrap_or_default();
        exec(input);
    }
}

fn help(_: &str) {
    println!("available commands:");
    println!("  exec <filename> [args...] - fork and exec in child");
    println!("  exit - exit the shell");
    println!("  help - show this message");
}

fn exec(input: String) {
    let cmd;
    let args;

    let items: Vec<&str> = input.split(" ").collect();
    if items.is_empty() {
        return;
    }

    if items.len() > 1 {
        cmd = items[0].trim().to_string();
        args = items[1..].join(" ");
    } else {
        cmd = input.trim().to_string();
        args = "".to_string();
    }

    match cmd.as_str() {
        "help" => help(&args),
        "exec" => tools::exec::run(&args),
        "exit" => process::exit(0),
        _ => {
            let mut path = format!("/init/bin/{}", cmd);
            let file = File::open(path.clone());
            if file.is_ok() {
                drop(file);
                path.push(' ');
                path.push_str(&args);
                tools::exec::run(&path)
            } else {
                println!("unknown command: {}", cmd);
            }
        },
    };
}
