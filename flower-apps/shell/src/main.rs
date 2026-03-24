#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use flower_libc::{print, println, std, tty};

mod tools;

const BUFFER_SIZE: usize = 64;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    tools::exec::run("/init/bin/fetch");

    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        print!(">");

        let len = tty::read_line(&mut buf);
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

fn help(_: &str) -> i32 {
    println!("available commands:");
    println!("  exec <filename> [args...] - fork and exec in child");
    0
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

    let ret_code = match cmd.as_str() {
        "help" => help(&args),
        "exec" => tools::exec::run(&args),
        _ => {
            let mut path = format!("/init/bin/{}", cmd);
            let file_fd = std::open(path.as_bytes(), 0, 0);

            if file_fd > 0 {
                std::close(file_fd as u64);
                path.push(' ');
                path.push_str(&args);
                tools::exec::run(&path)
            } else {
                println!("unknown command: {}", cmd);
                0
            }
        },
    };

    println!("command exited with code: {}", ret_code);
}
