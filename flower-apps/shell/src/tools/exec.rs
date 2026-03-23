use flower_libc::{println, std};

pub fn run(args: &str) -> i32 {
    if args.trim().is_empty() {
        println!("usage: exec <filename>");
        return -1;
    }

    let target = args.trim().as_bytes();

    let pid = std::fork();
    if pid < 0 {
        println!("fork failed: {}", pid);
        return -1;
    }

    if pid == 0 {
        let rc = std::execve(target, 0, 0);
        println!("execve failed: {}", rc);
        std::exit(127);
    }

    println!("forked child pid {}", pid);
    0
}
