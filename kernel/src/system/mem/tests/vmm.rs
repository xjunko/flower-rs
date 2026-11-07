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

use crate::arch::addr::KPhysAddr;
use crate::arch::generic::paging::MapFlags;
use crate::arch::layout::{PAGE_SIZE, USER_MMAP_BASE};
use crate::system::mem::pmm;
use crate::system::mem::vma::{Vma, VmaKind, VmaMap};
use crate::system::mem::vmm::AddressSpace;

pub fn test_vmm_map_alloc() {
    let space = AddressSpace::current();
    let before = pmm::free_pages().expect("pmm not installed");
    let virt = AddressSpace::reserve_virt(PAGE_SIZE).expect("valloc exhausted");

    let phys = space.map_page_alloc(virt, MapFlags::WRITE).expect("map failed");

    assert!(space.is_mapped(virt));
    assert_eq!(space.translate(virt), Some(phys));
    assert_eq!(pmm::free_pages(), Some(before - 1));

    // the frame is zeroed and writable through the hhdm.
    unsafe {
        let ptr = AddressSpace::phys_to_virt(phys).as_mut_ptr::<u8>();
        assert_eq!(*ptr, 0);
        assert_eq!(*ptr.add(PAGE_SIZE - 1), 0);
        *ptr = 0xAA;
        assert_eq!(*ptr, 0xAA);
    }

    let unmapped = space.unmap_page(virt).expect("unmap failed");
    assert_eq!(unmapped, phys);
    assert!(!space.is_mapped(virt));
    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_vmm_shared_page() {
    let space = AddressSpace::current();
    let before = pmm::free_pages().expect("pmm not installed");
    let frame = pmm::alloc_zeroed().expect("pmm out of memory");
    assert_eq!(pmm::free_pages(), Some(before - 1));

    let virt = AddressSpace::reserve_virt(PAGE_SIZE).expect("valloc exhausted");

    space
        .map_page_shared(virt, KPhysAddr::new(frame), MapFlags::WRITE)
        .expect("shared map failed");

    assert_eq!(space.translate(virt), Some(KPhysAddr::new(frame)));
    // the mapping owns its own reference, so the frame is still allocated.
    assert_eq!(pmm::free_pages(), Some(before - 1));

    space.unmap_page(virt).expect("unmap failed");
    // the caller still holds its reference.
    assert_eq!(pmm::free_pages(), Some(before - 1));

    pmm::free(frame);
    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_vmm_raw_page() {
    let space = AddressSpace::current();
    let before = pmm::free_pages().expect("pmm not installed");
    let frame = pmm::alloc_zeroed().expect("pmm out of memory");
    let virt = AddressSpace::reserve_virt(PAGE_SIZE).expect("valloc exhausted");

    space
        .map_page_raw(virt, KPhysAddr::new(frame), MapFlags::WRITE)
        .expect("raw map failed");

    assert_eq!(space.translate(virt), Some(KPhysAddr::new(frame)));

    let unmapped = space.unmap_page_raw(virt).expect("raw unmap failed");
    assert_eq!(unmapped, KPhysAddr::new(frame));
    // raw mappings must not touch the frame's reference count.
    assert_eq!(pmm::free_pages(), Some(before - 1));

    pmm::free(frame);
    assert_eq!(pmm::free_pages(), Some(before));
}

pub fn test_vmm_alloc_virt_range() {
    const PAGES: usize = 3;

    let space = AddressSpace::current();
    let before = pmm::free_pages().expect("pmm not installed");

    let virt = space
        .alloc_virt(PAGES * PAGE_SIZE, MapFlags::WRITE)
        .expect("alloc_virt failed");

    assert_eq!(pmm::free_pages(), Some(before - PAGES));

    for page in 0..PAGES {
        let addr = virt + (page * PAGE_SIZE) as u64;
        assert!(space.is_mapped(addr), "page {} not mapped", page);
    }

    space.free_virt(virt, PAGES * PAGE_SIZE).expect("free_virt failed");

    for page in 0..PAGES {
        let addr = virt + (page * PAGE_SIZE) as u64;
        assert!(!space.is_mapped(addr), "page {} still mapped", page);
    }

    assert_eq!(pmm::free_pages(), Some(before));

    // the reservation must be reusable, not leaked.
    let again = AddressSpace::reserve_virt(PAGES * PAGE_SIZE)
        .expect("valloc did not reclaim the freed range");
    assert_eq!(again, virt, "valloc did not reuse the freed address");
    AddressSpace::release_virt(again, PAGES * PAGE_SIZE)
        .expect("release failed");
}

pub fn test_valloc_reuse() {
    let first =
        AddressSpace::reserve_virt(PAGE_SIZE).expect("valloc exhausted");
    AddressSpace::release_virt(first, PAGE_SIZE).expect("release failed");

    let second =
        AddressSpace::reserve_virt(PAGE_SIZE).expect("valloc exhausted");
    assert_eq!(first, second, "valloc did not reuse the freed page");

    AddressSpace::release_virt(second, PAGE_SIZE).expect("release failed");
}

pub fn test_valloc_coalesce() {
    let first =
        AddressSpace::reserve_virt(2 * PAGE_SIZE).expect("valloc exhausted");
    let second =
        AddressSpace::reserve_virt(2 * PAGE_SIZE).expect("valloc exhausted");

    assert_eq!(
        second.as_u64(),
        first.as_u64() + (2 * PAGE_SIZE) as u64,
        "valloc did not hand out adjacent ranges"
    );

    AddressSpace::release_virt(first, 2 * PAGE_SIZE).expect("release failed");
    AddressSpace::release_virt(second, 2 * PAGE_SIZE).expect("release failed");

    // both halves must coalesce back into one four-page range.
    let merged = AddressSpace::reserve_virt(4 * PAGE_SIZE)
        .expect("valloc did not coalesce free ranges");
    assert_eq!(
        merged, first,
        "coalesced range did not start at the first page"
    );

    AddressSpace::release_virt(merged, 4 * PAGE_SIZE).expect("release failed");
}

pub fn test_vma_map_operations() {
    let flags = MapFlags::READ | MapFlags::WRITE | MapFlags::USER;
    let mut map = VmaMap::new();

    map.add_anonymous(0x1000, 0x5000, flags).expect("add failed");
    map.insert(Vma {
        start: 0x8000,
        end: 0xa000,
        flags,
        kind: VmaKind::Anonymous,
    })
    .expect("insert failed");

    assert!(map.find(0x1000).is_some());
    assert!(map.find(0x4fff).is_some());
    assert!(map.find(0x5000).is_none());

    let removed = map.remove_range(0x2000, 0x9000);
    assert_eq!(removed.len(), 2, "expected both regions to be split");

    assert!(map.find(0x1000).is_some(), "left remainder missing");
    assert!(map.find(0x2000).is_none(), "removed region still present");
    assert!(map.find(0x9000).is_some(), "right remainder missing");
    assert!(map.find(0x9fff).is_some());
    assert!(map.find(0xa000).is_none());

    // merging adjacent anonymous regions
    map.add_anonymous(0x1000, 0x3000, flags).expect("merge failed");
    map.add_anonymous(0x3000, 0x4000, flags).expect("merge failed");
    assert!(map.find(0x2000).is_some());

    let overlapping = map.insert(Vma {
        start: 0x1000,
        end: 0x2000,
        flags,
        kind: VmaKind::Anonymous,
    });
    assert!(overlapping.is_err(), "overlapping insert was accepted");
}

pub fn test_vma_adjacent_flags_stay_separate() {
    let rx = MapFlags::READ | MapFlags::EXEC | MapFlags::USER;
    let rw = MapFlags::READ | MapFlags::WRITE | MapFlags::USER;

    let mut map = VmaMap::new();
    map.add_anonymous(0x1000, 0x2000, rx).expect("add failed");
    map.add_anonymous(0x2000, 0x3000, rw).expect("add failed");

    // an r-x segment immediately followed by an rw segment must not be merged
    // into one rwx region.
    assert_eq!(map.len(), 2, "adjacent regions with different flags merged");
    assert!(
        !map.find(0x1000)
            .expect("missing rx region")
            .flags
            .intersects(MapFlags::WRITE)
    );
    assert!(
        map.find(0x2000)
            .expect("missing rw region")
            .flags
            .contains(MapFlags::WRITE)
    );

    // regions that share a page have to merge (and union their flags).
    let mut overlap = VmaMap::new();
    overlap.add_anonymous(0x1000, 0x3000, rw).expect("add failed");
    overlap.add_anonymous(0x2000, 0x4000, rx).expect("overlap merge failed");
    assert_eq!(overlap.len(), 1, "overlapping regions did not merge");
    assert!(
        overlap
            .find(0x2000)
            .expect("missing region")
            .flags
            .contains(MapFlags::WRITE | MapFlags::EXEC)
    );
}

pub fn test_vma_partial_shmem_fork() {
    let before = pmm::free_pages().expect("pmm not installed");
    let flags = MapFlags::READ | MapFlags::WRITE | MapFlags::USER;
    let first = USER_MMAP_BASE;
    let second = USER_MMAP_BASE + PAGE_SIZE as u64;

    let shmem = crate::system::mem::vma::Shmem::new(4 * PAGE_SIZE);

    {
        let mut parent = AddressSpace::new().expect("failed to create space");

        // map only the first page of a four-page object, then something else
        // directly after it. rebinding must not touch the second page.
        parent
            .map_user_shmem(first, PAGE_SIZE, flags, shmem.clone())
            .expect("partial shared map failed");
        parent
            .map_user_anonymous(second, PAGE_SIZE, flags)
            .expect("anonymous map failed");

        let shared = parent
            .translate(crate::arch::addr::KVirtAddr::new(first))
            .expect("shared page not mapped");
        let private = parent
            .translate(crate::arch::addr::KVirtAddr::new(second))
            .expect("private page not mapped");

        unsafe {
            core::ptr::write_bytes(
                AddressSpace::phys_to_virt(shared).as_mut_ptr::<u8>(),
                0x5A,
                PAGE_SIZE,
            );
            core::ptr::write_bytes(
                AddressSpace::phys_to_virt(private).as_mut_ptr::<u8>(),
                0xCD,
                PAGE_SIZE,
            );
        }

        let child = parent.clone_user().expect("fork clone failed");

        // the mapped prefix is shared with the parent...
        assert_eq!(
            child.translate(crate::arch::addr::KVirtAddr::new(first)),
            Some(shared)
        );

        // ...but the adjacent private page is still a private copy: it must
        // keep its own frame and its contents, not become shmem page 1.
        let child_private = child
            .translate(crate::arch::addr::KVirtAddr::new(second))
            .expect("child lost the private page");
        assert_ne!(
            child_private, private,
            "fork did not copy the adjacent private page"
        );

        unsafe {
            let ptr = AddressSpace::phys_to_virt(child_private).as_ptr::<u8>();
            assert_eq!(*ptr, 0xCD, "private page contents were clobbered");
        }

        // the object only owns its populated first page.
        assert!(shmem.is_populated(0));
        assert!(!shmem.is_populated(1), "fork populated an unmapped page");
        assert!(!shmem.is_populated(2));
        assert!(!shmem.is_populated(3));
    }

    // release the object handle so the object's own page references go away.
    drop(shmem);

    assert_eq!(
        pmm::free_pages(),
        Some(before),
        "partial shared mapping leaked frames across fork and teardown"
    );
}

pub fn test_vma_shared_fork() {
    let before = pmm::free_pages().expect("pmm not installed");

    {
        let mut parent = AddressSpace::new().expect("failed to create space");
        let start = USER_MMAP_BASE;

        parent
            .map_user_shared(
                start,
                2 * PAGE_SIZE,
                MapFlags::READ | MapFlags::WRITE | MapFlags::USER,
            )
            .expect("shared map failed");

        let first = parent
            .translate(crate::arch::addr::KVirtAddr::new(start))
            .expect("first page not mapped");
        let second = parent
            .translate(crate::arch::addr::KVirtAddr::new(
                start + PAGE_SIZE as u64,
            ))
            .expect("second page not mapped");

        unsafe {
            core::ptr::write_bytes(
                AddressSpace::phys_to_virt(first).as_mut_ptr::<u8>(),
                0xAB,
                PAGE_SIZE,
            );
        }

        let child = parent.clone_user().expect("fork clone failed");

        // shared pages must be the same physical frames in both processes.
        assert_eq!(
            child.translate(crate::arch::addr::KVirtAddr::new(start)),
            Some(first)
        );
        assert_eq!(
            child.translate(crate::arch::addr::KVirtAddr::new(
                start + PAGE_SIZE as u64,
            )),
            Some(second)
        );

        unsafe {
            let ptr = AddressSpace::phys_to_virt(first).as_ptr::<u8>();
            assert_eq!(*ptr, 0xAB);
        }
    }

    assert_eq!(
        pmm::free_pages(),
        Some(before),
        "shared mapping leaked frames across fork and teardown"
    );
}
