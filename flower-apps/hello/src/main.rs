#![no_std]
#![no_main]

use flower_libc::std;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // stdout write
    std::write(1, b"hello from userspace rust!\n");

    // timing test
    let mut i = 0;
    while i < 5 {
        std::sleep(16);
        i += 1;
        std::write(1, b"quick sleep!\n");
    }

    // read file test
    let file_fd = std::open(b"/init/hello.txt\0", 0, 0);
    std::write(1, b"opened file /init/hello.txt\n");
    if file_fd > 0 {
        std::write(1, b"reading file /init/hello.txt\n");
        let mut buf = [0u8; 128];
        let n = std::read(file_fd as u64, &mut buf);
        if n > 0 {
            std::write(1, &buf[..n]);
        }
        std::write(1, b"closing file /init/hello.txt\n");
        std::close(file_fd as u64);
    } else {
        std::write(1, b"failed to open file /init/hello.txt\n");
    }

    // should properly exit
    std::exit(0);
}
