#![no_std]
#![no_main]

use flower_libc::{std, tty};

mod tools;

const BUFFER_SIZE: usize = 256;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        std::write(1, b"> ");

        let len = tty::read_line(&mut buf);
        if len == 0 {
            continue;
        }
        buf[len..BUFFER_SIZE].fill(0);
        exec(&buf);
    }
}

fn exec(cmd: &[u8]) {
    let mut parts = cmd.splitn(2, |&b| b == b' ');
    let cmd = parts.next().unwrap_or(b"");
    let args = parts.next().unwrap_or(b"");

    let _ = {
        match cmd {
            b"cat" => tools::cat::read(args),
            b"pcm" => tools::pcm::play(args),
            _ => {
                std::write(1, b"unknown command\n");
                -1
            },
        }
    };
    std::write(1, b"command exited with code: {}\n");
}
