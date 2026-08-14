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

use crate::print;
use crate::system::vfs::devfs::{DevFile, DevFs};
use crate::system::vfs::error::VfsResult;

fn stderr_write(buf: &[u8]) -> VfsResult<usize> {
    // hack: just print to stdout for now
    for &byte in buf {
        print!("{}", byte as char);
    }
    Ok(buf.len())
}

pub(crate) fn bind(dev: &mut DevFs) {
    dev.bind(DevFile::new(
        "/stderr".to_string(),
        None,
        Some(stderr_write),
        None,
    ));
}
