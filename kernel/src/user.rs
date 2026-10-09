use crate::system::vfs::VFSFilelike;
use crate::system::{self};

const SHELL_PATH: &str = "/bin/shell";
pub fn entry() {
    if let Ok(file) = system::vfs::open(SHELL_PATH, 0) {
        match file {
            VFSFilelike::File(f) => {
                let metadata = f.metadata().expect("invalid metadata");
                let mut buffer = alloc::vec![0u8; metadata.size ];
                f.read(&mut buffer).expect("failed to read file");
                system::proc::user::spawn_elf("shell", &buffer)
                    .expect("failed to spawn shell process");
            },
            _ => {
                log::error!("{} is not a regular file", SHELL_PATH);
            },
        }
    } else {
        log::error!("failed to open file {}", SHELL_PATH);
    }
}
