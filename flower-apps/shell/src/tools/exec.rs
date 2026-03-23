use flower_libc::{println, std};

pub fn run(args: &str) -> i32 {
    const MAX_PATH: usize = 256;
    const MAX_ARGS: usize = 16;
    const MAX_ARG_LEN: usize = 128;

    let input = args.trim();
    if input.is_empty() {
        println!("usage: exec <filename> [args...]");
        return -1;
    }

    let mut path_c = [0u8; MAX_PATH];
    let mut arg_storage = [[0u8; MAX_ARG_LEN]; MAX_ARGS];
    let mut argv = [core::ptr::null::<core::ffi::c_char>(); MAX_ARGS + 1];

    let mut tokens = input.split_whitespace();
    let path = match tokens.next() {
        Some(path) => path,
        None => {
            println!("usage: exec <filename> [args...]");
            return -1;
        },
    };

    if path.len() + 1 > MAX_PATH {
        println!("path too long");
        return -1;
    }
    path_c[..path.len()].copy_from_slice(path.as_bytes());
    path_c[path.len()] = 0;

    let mut argc = 0usize;
    if path.len() + 1 > MAX_ARG_LEN {
        println!("arg too long: {}", path);
        return -1;
    }
    arg_storage[argc][..path.len()].copy_from_slice(path.as_bytes());
    arg_storage[argc][path.len()] = 0;
    argv[argc] = arg_storage[argc].as_ptr() as *const core::ffi::c_char;
    argc += 1;

    for token in tokens {
        if argc >= MAX_ARGS {
            println!("too many args (max {})", MAX_ARGS);
            return -1;
        }
        if token.len() + 1 > MAX_ARG_LEN {
            println!("arg too long: {}", token);
            return -1;
        }

        arg_storage[argc][..token.len()].copy_from_slice(token.as_bytes());
        arg_storage[argc][token.len()] = 0;
        argv[argc] = arg_storage[argc].as_ptr() as *const core::ffi::c_char;
        argc += 1;
    }
    argv[argc] = core::ptr::null();

    let pid = std::fork();
    if pid < 0 {
        println!("fork failed: {}", pid);
        return -1;
    }

    if pid == 0 {
        let rc =
            std::execve(&path_c[..path.len() + 1], argv.as_ptr() as u64, 0);
        println!("execve failed: {}", rc);
        std::exit(127);
    }

    std::sleep(100);

    0
}
