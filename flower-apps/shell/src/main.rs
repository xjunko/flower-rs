#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use flower_libc::sys::fs;
use flower_libc::{io, print, println, process};

mod tools;

const BUFFER_SIZE: usize = 64;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

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
            let file_fd = fs::open(path.as_bytes(), 0, 0);

            if file_fd > 0 {
                fs::close(file_fd as u64);
                path.push(' ');
                path.push_str(&args);
                tools::exec::run(&path)
            } else {
                println!("unknown command: {}", cmd);
            }
        },
    };
}
