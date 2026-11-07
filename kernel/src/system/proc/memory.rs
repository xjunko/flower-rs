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

//! user memory bookkeeping.
//!
//! the address space already owns the mappings and the vma list; this module
//! only carries the small amount of state that has to survive across syscalls:
//! where the heap lives and where its program break currently is.

use crate::system::mem::vmm::AddressSpace;

/// a half-open range of user virtual addresses, `[start, end)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KUserRange {
    start: u64,
    end: u64,
}

impl KUserRange {
    pub const fn new(start: u64, end: u64) -> Self { Self { start, end } }

    pub const fn start(&self) -> u64 { self.start }

    pub const fn end(&self) -> u64 { self.end }

    pub const fn len(&self) -> u64 { self.end - self.start }

    pub const fn is_empty(&self) -> bool { self.start == self.end }

    pub const fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }
}

/// the runtime state of a user heap: its reserved range and current break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KUserHeap {
    range: KUserRange,
    brk: u64,
}

impl KUserHeap {
    pub const fn new(range: KUserRange, brk: u64) -> Self {
        Self { range, brk }
    }

    pub const fn range(&self) -> KUserRange { self.range }

    /// first address of the reserved heap.
    pub const fn base(&self) -> u64 { self.range.start() }

    /// one past the last address the heap may grow into.
    pub const fn limit(&self) -> u64 { self.range.end() }

    /// the current program break.
    pub const fn brk(&self) -> u64 { self.brk }

    /// moves the program break if it stays inside the reserved range.
    /// returns `false` and leaves the break untouched otherwise.
    pub fn set_brk(&mut self, brk: u64) -> bool {
        if brk < self.base() || brk > self.limit() {
            return false;
        }

        self.brk = brk;
        true
    }
}

/// a loaded user image, ready to be installed into a process.
///
/// the loader maps the page containing `program_break` (the first heap page),
/// so the initial break is one page into `heap`.
pub struct KUserImage {
    pub address_space: AddressSpace,
    pub entry: u64,
    pub stack_pointer: u64,
    pub heap: KUserRange,
    pub program_break: u64,
}

/// the result of installing a [`KUserImage`] into an existing process.
///
/// the previous address space is handed back rather than dropped so the caller
/// can switch page tables first: freeing the root that is still loaded in cr3
/// would be a use-after-free.
pub struct KImageLoad {
    pub previous_address_space: Option<AddressSpace>,
    pub entry: u64,
    pub stack_pointer: u64,
}
