use alloc::string::ToString;

use crate::drivers;
use crate::system::vfs::devfs::{DevFS, DevFile};

struct DevFSAudio;

const AC97_WRITE_CHUNK_SIZE: usize = 4092;
const AC97_FRAME_SIZE: usize = 4;

fn audio_read(_buf: &mut [u8]) -> usize { unimplemented!() }

fn audio_write(_buf: &[u8]) -> usize {
    let mut guard = drivers::pci::devices::ac97::get_driver();
    if let Some(driver) = guard.as_mut() {
        let mut total_written = 0;

        let mut chunks = _buf.chunks_exact(AC97_WRITE_CHUNK_SIZE);
        for chunk in &mut chunks {
            while !driver.can_write() {
                core::hint::spin_loop();
            }

            let written = driver.write_buffer(chunk);
            if written == 0 {
                return total_written;
            }

            total_written += written;
        }

        let tail = chunks.remainder();
        let aligned_len = tail.len() - (tail.len() % AC97_FRAME_SIZE);
        if aligned_len > 0 {
            while !driver.can_write() {
                core::hint::spin_loop();
            }

            let written = driver.write_buffer(&tail[..aligned_len]);
            if written == 0 {
                return total_written;
            }

            total_written += written;
        }

        let remainder = &tail[aligned_len..];
        if !remainder.is_empty() {
            while !driver.can_write() {
                core::hint::spin_loop();
            }

            let mut padded = [0u8; AC97_FRAME_SIZE];
            padded[..remainder.len()].copy_from_slice(remainder);

            if driver.write_buffer(&padded) == 0 {
                return total_written;
            }

            total_written += remainder.len();
        }

        total_written
    } else {
        0
    }
}

pub fn install(dev: &mut DevFS) {
    dev.bind(DevFile {
        path: "/audio".to_string(),
        _read: Some(audio_read),
        _write: Some(audio_write),
    });
}
