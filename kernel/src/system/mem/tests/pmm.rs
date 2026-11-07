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

use alloc::vec::Vec;

use crate::arch::layout::PAGE_SIZE;
use crate::system::mem::pmm;

pub fn test_pmm_roundtrip() {
    let before = pmm::free_pages().expect("pmm not installed");
    let addr = pmm::alloc().expect("pmm out of memory");

    assert_eq!(addr % PAGE_SIZE as u64, 0);
    assert_eq!(pmm::free_pages(), Some(before - 1));
    assert!(pmm::is_usable_address(addr));

    pmm::free(addr);
    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_pmm_no_overlap() {
    const FRAMES: usize = 256;

    let mut frames = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        frames.push(pmm::alloc().expect("pmm out of memory"));
    }

    let mut sorted = frames.clone();
    sorted.sort_unstable();
    sorted.dedup();

    assert_eq!(sorted.len(), FRAMES, "pmm handed out a frame twice");

    for frame in sorted.iter() {
        assert_eq!(frame % PAGE_SIZE as u64, 0);
        assert!(pmm::is_usable_address(*frame));
    }

    for frame in frames {
        pmm::free(frame);
    }
}

pub fn test_pmm_reuse_after_free() {
    let before = pmm::free_pages().expect("pmm not installed");

    let addr = pmm::alloc().expect("pmm out of memory");
    pmm::free(addr);
    assert_eq!(pmm::free_pages(), Some(before));

    // the freed frame must be allocatable again (and still be valid).
    let again = pmm::alloc().expect("pmm out of memory");
    assert_eq!(again % PAGE_SIZE as u64, 0);
    assert!(pmm::is_usable_address(again));

    pmm::free(again);
    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_pmm_double_free() {
    let before = pmm::free_pages().expect("pmm not installed");

    let addr = pmm::alloc().expect("pmm out of memory");
    pmm::free(addr);
    assert_eq!(pmm::free_pages(), Some(before));

    // must be rejected without changing the free count.
    pmm::free(addr);
    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_pmm_invalid_free() {
    let before = pmm::free_pages().expect("pmm not installed");

    // unaligned, out of range, and reserved/low memory.
    pmm::free(0x123);
    pmm::free(0xFFFF_FFFF_FFFF_F000);
    pmm::free(0);

    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_pmm_refcount() {
    let before = pmm::free_pages().expect("pmm not installed");
    let addr = pmm::alloc().expect("pmm out of memory");

    assert_eq!(pmm::retain(addr), Ok(()));

    // refcount 2 -> 1: still allocated.
    pmm::free(addr);
    assert_eq!(pmm::free_pages(), Some(before - 1));

    // refcount 1 -> 0: returned to the free list.
    pmm::free(addr);
    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_pmm_stats() {
    let total = pmm::total_pages().expect("pmm not installed");
    let usable = pmm::usable_pages().expect("pmm not installed");
    let free = pmm::free_pages().expect("pmm not installed");

    assert!(total >= usable, "usable pages exceed total pages");
    assert!(usable >= free, "free pages exceed usable pages");

    let max = pmm::max_phys_address().expect("pmm not installed");
    assert_eq!(max, (total * PAGE_SIZE) as u64);
}
