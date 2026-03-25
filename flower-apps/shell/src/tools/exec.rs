use alloc::string::{String, ToString};
use alloc::vec::Vec;

use flower_libc::{println, process};

fn run_inner(args: &str, print_exit_status: bool) {
    let input = args.trim();
    if input.is_empty() {
        println!("usage: exec <filename> [args...]");
        return;
    }

    let mut tokens = input.split_whitespace();
    let path = match tokens.next() {
        Some(path) => path,
        None => {
            println!("usage: exec <filename> [args...]");
            return;
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

    let pid = process::fork();
    if pid < 0 {
        println!("fork failed: {}", pid);
        return;
    }

    if pid == 0 {
        let rc = process::execve(path_c.as_bytes(), argv.as_ptr() as u64, 0);
        println!("execve failed: {}", rc);
        process::exit(127);
    }

    let status = process::waitpid(pid as u64);
    if status < 0 {
        println!("waitpid failed");
        return;
    }

    if print_exit_status {
        println!("process exited with code {}", status);
    }
}

pub fn run(args: &str) { run_inner(args, true); }

pub fn run_quiet(args: &str) { run_inner(args, false); }
