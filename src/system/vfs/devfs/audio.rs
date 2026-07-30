use alloc::string::ToString;

use crate::devices;
use crate::system::vfs::devfs::{DevFS, DevFile};

struct DevFSAudio;

const AC97_WRITE_CHUNK_SIZE: usize = 4092;
const AC97_FRAME_SIZE: usize = 4;

fn audio_read(_offset: usize, _buf: &mut [u8]) -> usize { unimplemented!() }

fn audio_write(buf: &[u8]) -> usize {
    if devices::pci::drivers::ac97::ready() {
        let mut total_written = 0;
        let mut chunks = buf.chunks_exact(AC97_WRITE_CHUNK_SIZE);

        for chunk in &mut chunks {
            while devices::pci::drivers::ac97::busy() {
                core::hint::spin_loop();
            }

            let written = devices::pci::drivers::ac97::write(chunk);
            if written == 0 {
                return total_written;
            }
            total_written += written;
        }

        // tail
        let tail = chunks.remainder();
        let aligned_len = tail.len() - (tail.len() % AC97_FRAME_SIZE);
        if aligned_len > 0 {
            while devices::pci::drivers::ac97::busy() {
                core::hint::spin_loop();
            }
            let written =
                devices::pci::drivers::ac97::write(&tail[..aligned_len]);
            if written == 0 {
                return total_written;
            }
            total_written += written;
        }

        // remainder
        let remainder = &tail[aligned_len..];
        if !remainder.is_empty() {
            while devices::pci::drivers::ac97::busy() {
                core::hint::spin_loop();
            }

            let mut padded = [0u8; AC97_FRAME_SIZE];
            padded[..remainder.len()].copy_from_slice(remainder);

            if devices::pci::drivers::ac97::write(&padded) == 0 {
                return total_written;
            }

            total_written += remainder.len();
        }

        total_written
    } else {
        0
    }
}

fn audio_volume_write(_buf: &[u8]) -> usize { todo!() }

pub fn install(dev: &mut DevFS) {
    dev.bind(DevFile::new(
        "/audio".to_string(),
        Some(audio_read),
        Some(audio_write),
        None,
    ));

    dev.bind(DevFile::new(
        "/audio/volume".to_string(),
        None,
        Some(audio_volume_write),
        None,
    ));
}
