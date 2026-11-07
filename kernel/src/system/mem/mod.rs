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

use crate::arch;
use crate::boot::KBootInfo;

pub mod fault;
pub mod heap;
pub mod kstack;
pub mod pmm;
pub mod vma;
pub mod vmm;

mod tests;

pub fn install() {
    let boot_info = KBootInfo::get();
    let memory_map = boot_info.memory.lock();

    log::debug!("==== MEMORY MAP ====");
    memory_map.iter().for_each(|x| {
        log::debug!(
            "[{:#x} - {:#x}] {:?}",
            x.address,
            x.address + x.length as u64,
            x.usage
        );
    });
    log::debug!("========");
    drop(memory_map);

    arch::generic::addr::HHDM.call_once(|| boot_info.hhdm.unwrap());
    pmm::install();
    vmm::install();
    heap::install().expect("failed to install heap");
}

pub fn self_test() { tests::start(); }
