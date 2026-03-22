use flower_libc::{println, std};

pub fn read(args: &str) -> i32 {
    if args.is_empty() {
        println!("usage: cat <filename>");
        return -1;
    }

    let file_fd = std::open(args.as_bytes(), 0, 0);
    if file_fd < 0 {
        println!("failed to open file");
        return -1;
    }

    let mut buffer = [0u8; 1024];
    loop {
        let read_bytes = std::read(file_fd as u64, &mut buffer);
        if read_bytes <= 0 {
            break;
        }
        std::write(1, &buffer[..read_bytes as usize]);
    }
    std::close(file_fd as u64);

    0
}
