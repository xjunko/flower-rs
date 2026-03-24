use crate::{println, system};

fn logo() {
    println!(
        "flower@vocachuds
------------
Kernel:   {} [v{}]
Memory:   {}/{}MB
",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        system::mem::heap::used_memory() / 1024 / 1024,
        system::mem::heap::heap_capacity() / 1024 / 1024
    )
}

const SHELL_PATH: &str = "/init/bin/shell";
pub fn entry() {
    logo();

    if let Ok(file) = system::vfs::open(SHELL_PATH, 0) {
        let metadata = file.metadata().expect("invalid metadata");
        let mut buffer = alloc::vec![0u8; metadata.size ];
        file.read(&mut buffer).expect("failed to read file");
        system::proc::spawn_elf("shell", &buffer)
            .expect("failed to spawn shell process");
    } else {
        log::error!("failed to open file {}", SHELL_PATH);
    }
}
