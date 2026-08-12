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

use alloc::format;
use alloc::string::ToString;

use crate::system::vfs::devfs::{DevFS, DevFile};
use crate::{arch, memory};

fn meminfo_read(offset: usize, buf: &mut [u8]) -> usize {
    let mem_total = memory::pmm::usable_pages().unwrap_or(0)
        * arch::x86_64::layout::PAGE_SIZE;
    let mem_free = memory::pmm::free_pages().unwrap_or(0)
        * arch::x86_64::layout::PAGE_SIZE;

    let mem_available = mem_free;
    let mem_used = mem_total.saturating_sub(mem_free);

    let meminfo = format!(
        "
MemTotal: {} kB
MemFree: {} kB
MemUsed: {} kB
MemAvailable: {} kB
",
        mem_total / 1024,
        mem_free / 1024,
        mem_used / 1024,
        mem_available / 1024,
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
        None,
    ));
    mnt.bind(DevFile::new(
        "/version".to_string(),
        Some(kernel_info),
        Some(_empty_write),
        None,
    ));
    mnt
}
