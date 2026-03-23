#![no_std]
#![no_main]

use flower_libc::{print, println, std, tty};

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

    let ret_code = match cmd
        .trim_matches(|c: char| c.is_whitespace() || c == '\0')
    {
        "help" => help(args),
        "exec" => tools::exec::run(args),
        _ => {
            // basically append cmd to /init/ and check if the file exists
            let mut path = [0u8; BUFFER_SIZE * 2 + 7];
            path[..6].copy_from_slice(b"/init/");
            let cmd =
                cmd.trim_matches(|c: char| c.is_whitespace() || c == '\0');
            let args =
                args.trim_matches(|c: char| c.is_whitespace() || c == '\0');

            let cmd_bytes = cmd.as_bytes();
            let cmd_len = core::cmp::min(cmd_bytes.len(), BUFFER_SIZE);
            let mut path_len = 6 + cmd_len;
            path[6..path_len].copy_from_slice(&cmd_bytes[..cmd_len]);

            let file_fd = std::open(&path[..path_len], 0, 0);

            if file_fd > 0 {
                // append args after the path if exists
                if !args.is_empty() {
                    path[path_len] = b' ';
                    path_len += 1;

                    let args_bytes = args.as_bytes();
                    let args_len =
                        core::cmp::min(args_bytes.len(), path.len() - path_len);
                    path[path_len..path_len + args_len]
                        .copy_from_slice(&args_bytes[..args_len]);
                    path_len += args_len;
                }

                let exec_args = str::from_utf8(&path[..path_len]).unwrap_or("");

                std::close(file_fd as u64);
                tools::exec::run(exec_args)
            } else {
                println!("unknown command: {}", cmd);
                -1
            }
        },
    };

    println!("command exited with code: {}", ret_code);
}
