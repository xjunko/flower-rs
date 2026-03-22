use flower_libc::std;

// fairly small buffer, let's just hope it's big enough.
const PCM_BUFFER: usize = 4096;

pub fn play(args: &[u8]) -> i32 {
    let file_fd = std::open(args, 0, 0);
    if file_fd < 0 {
        std::write(1, b"failed to open file\n");
        return -1;
    }

    let driver_fd = std::open(b"/dev/audio\0", 0, 0);
    if driver_fd < 0 {
        std::write(1, b"failed to open audio driver\n");
        return -1;
    }

    let mut buffer = [0u8; PCM_BUFFER];
    // let mut pcm_pos = 0;

    loop {
        let bytes_read = std::read(file_fd as u64, &mut buffer);
        if bytes_read <= 0 {
            break;
        }

        let mut total_written = 0;
        while total_written < bytes_read {
            let written = std::write(
                driver_fd as u64,
                &buffer[total_written as usize..bytes_read as usize],
            );
            if written < 0 {
                std::write(1, b"failed to write to audio driver\n");
                std::close(driver_fd as u64);
                std::close(file_fd as u64);
                return -1;
            }

            total_written += written;
        }
        // pcm_pos += bytes_read;
    }
    std::close(driver_fd as u64);
    std::close(file_fd as u64);

    0
}
