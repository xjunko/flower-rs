/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

use alloc::string::ToString;

use crate::devices;
use crate::system::vfs2::devfs::{DevFile, DevFs};
use crate::system::vfs2::error::{VfsError, VfsResult};

struct DevFSAudio;

const AC97_WRITE_CHUNK_SIZE: usize = 4092;
const AC97_FRAME_SIZE: usize = 4;

fn audio_read(_offset: usize, _buf: &mut [u8]) -> VfsResult<usize> {
    unimplemented!()
}

fn audio_write(buf: &[u8]) -> VfsResult<usize> {
    if devices::pci::drivers::ac97::ready() {
        let mut total_written = 0;
        let (chunks, remainder) = buf.as_chunks::<AC97_WRITE_CHUNK_SIZE>();

        for chunk in chunks {
            while devices::pci::drivers::ac97::busy() {
                core::hint::spin_loop();
            }

            let written = devices::pci::drivers::ac97::write(chunk);
            if written == 0 {
                return Err(VfsError::Unknown("AC97 write failed".to_string()));
            }
            total_written += written;
        }

        // tail
        let tail = remainder;
        let aligned_len = tail.len() - (tail.len() % AC97_FRAME_SIZE);
        if aligned_len > 0 {
            while devices::pci::drivers::ac97::busy() {
                core::hint::spin_loop();
            }
            let written =
                devices::pci::drivers::ac97::write(&tail[..aligned_len]);
            if written == 0 {
                return Err(VfsError::Unknown("AC97 write failed".to_string()));
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
                return Err(VfsError::Unknown("AC97 write failed".to_string()));
            }

            total_written += remainder.len();
        }

        Ok(total_written)
    } else {
        Ok(0)
    }
}

pub fn bind(dev: &mut DevFs) {
    dev.bind(DevFile::new(
        "/audio".to_string(),
        Some(audio_read),
        Some(audio_write),
        None,
    ));
}
