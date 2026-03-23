#![no_std]
#![no_main]

use flower_libc::{print, println, tty};

mod tools;

const BUFFER_SIZE: usize = 256;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        print!(">");

        let len = tty::read_line(&mut buf);
        if len == 0 {
            continue;
        }
        buf[len..BUFFER_SIZE].fill(0);
        exec(&buf[..len]);
    }
}

fn help(_: &str) -> i32 {
    println!("available commands:");
    println!("  cat <filename> - print the contents of a file");
    println!("  pcm <filename> - play a PCM audio file");
    0
}

fn exec(buf: &[u8]) {
    let (cmd, args) = str::from_utf8(buf)
        .map(|s| s.trim())
        .map(|s| match s.split_once(' ') {
            Some((c, a)) => (c, a.trim_start()),
            None => (s, ""),
        })
        .unwrap_or(("", ""));

    let ret_code = match cmd {
        "help" => help(args),
        "cat" => tools::cat::read(args),
        "pcm" => tools::pcm::play(args),
        _ => {
            println!("unknown command: {}", cmd);
            -1
        },
    };

    println!("command exited with code: {}", ret_code);
}
