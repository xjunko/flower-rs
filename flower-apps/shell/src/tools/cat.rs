use flower_libc::std;

pub fn read(args: &[u8]) -> i32 {
    let file_fd = std::open(args, 0, 0);
    if file_fd < 0 {
        std::write(1, b"failed to open file\n");
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
