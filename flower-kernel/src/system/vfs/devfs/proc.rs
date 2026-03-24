use alloc::format;
use alloc::string::ToString;

use crate::system::vfs::devfs::{DevFS, DevFile};
use crate::system::{self};

fn meminfo_read(offset: usize, buf: &mut [u8]) -> usize {
    let meminfo = format!(
        "
MemTotal: {} kB
MemAvailable: {} kB
",
        system::mem::heap::heap_capacity() / 1024,
        system::mem::heap::used_memory() / 1024,
    )
    .trim()
    .to_string();
    let bytes = meminfo.as_bytes();
    if offset >= bytes.len() {
        return 0;
    }

    let remaining = &bytes[offset..];
    let len = remaining.len().min(buf.len());
    buf[..len].copy_from_slice(&remaining[..len]);
    len
}

fn kernel_info(offset: usize, buf: &mut [u8]) -> usize {
    let meminfo = format!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
    .trim()
    .to_string();

    let bytes = meminfo.as_bytes();
    if offset >= bytes.len() {
        return 0;
    }

    let remaining = &bytes[offset..];
    let len = remaining.len().min(buf.len());
    buf[..len].copy_from_slice(&remaining[..len]);
    len
}

fn _empty_write(_buf: &[u8]) -> usize { 0 }

// this is really silly but it works...
pub fn create_procfs() -> DevFS {
    let mut mnt = DevFS::new();
    mnt.bind(DevFile::new(
        "/meminfo".to_string(),
        Some(meminfo_read),
        Some(_empty_write),
    ));
    mnt.bind(DevFile::new(
        "/version".to_string(),
        Some(kernel_info),
        Some(_empty_write),
    ));
    mnt
}
