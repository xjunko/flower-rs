use alloc::string::{String, ToString};
use alloc::vec::Vec;

use flower_libc::{println, std};

pub fn run(args: &str) -> i32 {
    let input = args.trim();
    if input.is_empty() {
        println!("usage: exec <filename> [args...]");
        return -1;
    }

    let mut tokens = input.split_whitespace();
    let path = match tokens.next() {
        Some(path) => path,
        None => {
            println!("usage: exec <filename> [args...]");
            return -1;
        },
    };

    let mut path_c = path.to_string();
    path_c.push('\0');

    let mut c_args: Vec<String> = Vec::new();
    c_args.push(path_c.clone());

    for token in tokens {
        let mut c_arg = token.to_string();
        c_arg.push('\0');
        c_args.push(c_arg);
    }

    let mut argv: Vec<*const core::ffi::c_char> =
        Vec::with_capacity(c_args.len() + 1);
    for c_arg in &c_args {
        argv.push(c_arg.as_ptr() as *const core::ffi::c_char);
    }
    argv.push(core::ptr::null());

    let pid = std::fork();
    if pid < 0 {
        println!("fork failed: {}", pid);
        return -1;
    }

    if pid == 0 {
        let rc = std::execve(path_c.as_bytes(), argv.as_ptr() as u64, 0);
        println!("execve failed: {}", rc);
        std::exit(127);
    }

    std::sleep(100);

    0
}
