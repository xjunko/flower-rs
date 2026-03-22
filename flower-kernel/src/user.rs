// static HELLO_ELF: &[u8] =
//     include_bytes!("../../target/x86_64-unknown-none/release/userspace-hello");

use crate::{println, system};

fn logo() {
    println!(
        "flower@vocachuds
------------
Kernel:   {}-{}
Memory:   {}/{}MB
",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        system::mem::pmm::free_pages().unwrap_or(0) * system::mem::PAGE_SIZE
            / 1024
            / 1024,
        system::mem::pmm::usable_pages().unwrap_or(0) * system::mem::PAGE_SIZE
            / 1024
            / 1024
    )
}

pub fn entry() {
    logo();

    // user-mode process test
    // system::proc::spawn_elf("hello", HELLO_ELF)
    //     .expect("failed to spawn elf process");

    // user-mode shell test
    // if let Ok(file) = system::vfs::open("/init/shell", 0) {
    //     let metadata = file.metadata().expect("invalid metadata");
    //     let mut buffer = alloc::vec![0u8; metadata.size ];
    //     file.read(&mut buffer).expect("failed to read file");
    //     system::proc::spawn_elf("shell", &buffer)
    //         .expect("failed to spawn shell process");
    // } else {
    //     println!("failed to open file /init/shell");
    // }
}
