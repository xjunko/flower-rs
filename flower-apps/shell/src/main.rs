#![no_std]
#![no_main]

use flower_libc::{print, println, tty};

mod tools;

const BUFFER_SIZE: usize = 64;

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
        exec(&buf);
    }
}

fn help(_: &str) -> i32 {
    println!("available commands:");
    println!("  exec <filename> [args...] - fork and exec in child");
    0
}

fn exec(buf: &[u8]) {
    let (cmd, args) = str::from_utf8(buf)
        .map(|s| {
            let s = s.trim();
            match s.split_once(' ') {
                Some((c, a)) => (c, a.trim_start()),
                None => (s, ""),
            }
        })
        .unwrap_or(("", ""));

    let ret_code =
        match cmd.trim_matches(|c: char| c.is_whitespace() || c == '\0') {
            "help" => help(args),
            "exec" => tools::exec::run(args),
            _ => {
                println!("unknown command: {}", cmd);
                -1
            },
        };

    println!("command exited with code: {}", ret_code);
}
