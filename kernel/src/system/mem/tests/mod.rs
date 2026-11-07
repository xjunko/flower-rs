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

mod heap;
mod pmm;
mod vmm;

pub fn start() {
    log::debug!("starting memory tests...");

    // pmm
    log::debug!("testing pmm...");
    {
        pmm::test_pmm_roundtrip();
        pmm::test_pmm_no_overlap();
        pmm::test_pmm_reuse_after_free();
        pmm::test_pmm_double_free();
        pmm::test_pmm_invalid_free();
        pmm::test_pmm_refcount();
        pmm::test_pmm_stats();
    }

    // vmm
    log::debug!("testing vmm...");
    {
        vmm::test_vmm_map_alloc();
        vmm::test_vmm_shared_page();
        vmm::test_vmm_raw_page();
        vmm::test_vmm_alloc_virt_range();
        vmm::test_valloc_reuse();
        vmm::test_valloc_coalesce();
        vmm::test_vma_map_operations();
        vmm::test_vma_adjacent_flags_stay_separate();
        vmm::test_vma_shared_fork();
        vmm::test_vma_partial_shmem_fork();
    }

    // heap
    log::debug!("testing heap...");
    {
        heap::test_heap_basic();
        heap::test_heap_vec();
        heap::test_heap_fragmentation();
        heap::test_heap_large();
        heap::test_heap_oom();
        heap::test_heap_stress();
    }
}
