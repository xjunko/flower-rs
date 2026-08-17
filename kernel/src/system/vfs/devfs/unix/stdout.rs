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

use flower_mono::kapi::stdout::{RIRIA_DISABLE_FB0, RIRIA_ENABLE_FB0};

use crate::print;
use crate::system::vfs::devfs::{DevFile, DevFs};
use crate::system::vfs::error::{VfsError, VfsResult};

fn stdout_write(buf: &[u8]) -> VfsResult<usize> {
    for &byte in buf {
        print!("{}", byte as char);
    }
    Ok(buf.len())
}

fn stdout_ioctl(cmd: u64, _arg: usize) -> VfsResult<usize> {
    // HACK: later on when we have a proper use for the /dev/fb0 that isnt just the terminal
    //       it should be possible for the user to disable the fb0 terminal completely, so it only goes out to the serial.
    match cmd {
        RIRIA_ENABLE_FB0 => {
            todo!()
        },
        RIRIA_DISABLE_FB0 => {
            todo!()
        },
        _ => Err(VfsError::InvalidArgument),
    }
}

pub(crate) fn bind(dev: &mut DevFs) {
    dev.bind(DevFile::new(
        "/stdout".to_string(),
        None,
        Some(stdout_write),
        None,
        Some(stdout_ioctl),
    ));
}
