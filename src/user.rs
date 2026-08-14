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

use crate::system::vfs::file::OpenFlags;
use crate::system::vfs::perm::Credentials;
use crate::system::{self};

const SHELL_PATH: &str = "/bin/shell";
pub fn entry() {
    if let Ok(file) =
        system::vfs::open(SHELL_PATH, OpenFlags::RDONLY, Credentials::ROOT)
    {
        let metadata = file.metadata().expect("invalid metadata");
        let mut buffer = alloc::vec![0u8; metadata.size ];
        file.read(buffer.as_mut_slice()).expect("failed to read file");
        system::proc::user::spawn_elf("shell", buffer.as_mut_slice())
            .expect("failed to spawn shell process");
    } else {
        log::error!("failed to open file {}", SHELL_PATH);
    }
}
