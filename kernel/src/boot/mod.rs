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

use spin::Once;
use spin::mutex::SpinMutex;

use crate::arch::addr::{KPhysAddr, KVirtAddr};

pub mod limine;

static BOOT_INFO: Once<KBootInfo> = Once::new();

#[derive(Debug)]
pub struct KBootInfo {
    pub files: &'static [KBootFile],
    pub memory: SpinMutex<&'static [KMemoryMap]>,
    pub highest_phys: Option<KPhysAddr>,
    pub rsdp_addr: Option<KPhysAddr>,
    pub hhdm: Option<KVirtAddr>,
}

impl KBootInfo {
    pub const fn new() -> Self {
        Self {
            files: &[],
            memory: SpinMutex::new(&[]),
            highest_phys: None,
            rsdp_addr: None,
            hhdm: None,
        }
    }

    pub fn register(self) { BOOT_INFO.call_once(|| self); }

    pub fn get() -> &'static Self { BOOT_INFO.get().expect("no boot info") }
}

#[derive(Clone, Copy, Debug)]
pub struct KBootFile {
    pub data: KPhysAddr,
    pub length: usize,
    pub path: &'static str,
}

impl KBootFile {
    pub const fn new() -> Self {
        Self { data: KPhysAddr::new(0), length: 0, path: "" }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub enum KMemoryMapType {
    #[default]
    Reserved,
    Reclaimable,
    Usable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct KMemoryMap {
    pub address: KPhysAddr,
    pub length: usize,
    pub usage: KMemoryMapType,
}

impl KMemoryMap {
    pub const fn empty() -> Self {
        Self {
            address: KPhysAddr::new(0),
            length: 0,
            usage: KMemoryMapType::Reserved,
        }
    }

    pub fn end(self) -> KPhysAddr { self.address + self.length as u64 }
}

pub fn install() { limine::install(); }
