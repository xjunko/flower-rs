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

use core::alloc::GlobalAlloc;

use linked_list_allocator::Heap;
use spin::Mutex;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::PageTableFlags;

use crate::arch::x86_64::layout::{
    KERNEL_HEAP_SIZE, KERNEL_HEAP_START, PAGE_SIZE,
};
use crate::memory;

struct Allocator;
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

struct AllocStateInner {
    heap: Option<Heap>,
    heap_size: usize,
}

static ALLOC_STATE: Mutex<AllocStateInner> =
    Mutex::new(AllocStateInner { heap: None, heap_size: 0 });

fn map_chunk(
    addr: VirtAddr,
    size: usize,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    let pages = size.div_ceil(PAGE_SIZE);

    for i in 0..pages {
        let page_addr = addr + (i * PAGE_SIZE) as u64;

        memory::vmm::page_map_alloc(page_addr, flags)?;
    }

    Ok(())
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        interrupts::without_interrupts(|| {
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_EXECUTE;

            let mut state = ALLOC_STATE.lock();

            if state.heap.is_none() {
                let base = VirtAddr::new(KERNEL_HEAP_START as u64);
                if map_chunk(base, KERNEL_HEAP_SIZE, flags).is_err() {
                    return core::ptr::null_mut();
                }
                state.heap_size = KERNEL_HEAP_SIZE;
                state.heap = Some(unsafe {
                    Heap::new(base.as_mut_ptr(), KERNEL_HEAP_SIZE)
                });
            }

            // there's a chance that this will loop forever
            // but let's just hope that doesn't happen
            loop {
                if let Some(heap) = state.heap.as_mut()
                    && let Ok(ptr) = heap.allocate_first_fit(layout)
                {
                    return ptr.as_ptr();
                }

                let addr =
                    VirtAddr::new((KERNEL_HEAP_START + state.heap_size) as u64);

                if let Err(reason) = map_chunk(addr, KERNEL_HEAP_SIZE, flags) {
                    log::error!(
                        "failed to map chunk for heap expansion: {}",
                        reason
                    );
                    return core::ptr::null_mut();
                }

                if let Some(heap) = state.heap.as_mut() {
                    unsafe {
                        heap.extend(KERNEL_HEAP_SIZE);
                    }
                }
                state.heap_size += KERNEL_HEAP_SIZE;
            }
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        interrupts::without_interrupts(|| {
            let mut state = ALLOC_STATE.lock();
            if let Some(heap) = state.heap.as_mut() {
                unsafe {
                    heap.deallocate(
                        core::ptr::NonNull::new(ptr).unwrap(),
                        layout,
                    );
                }
            }
        });
    }
}

pub fn install() -> Result<(), &'static str> { Ok(()) }

pub fn free_memory() -> usize {
    interrupts::without_interrupts(|| {
        let state = ALLOC_STATE.lock();
        state.heap.as_ref().map_or(0, |heap| heap.free())
    })
}

pub fn heap_capacity() -> usize {
    interrupts::without_interrupts(|| {
        let state = ALLOC_STATE.lock();
        state.heap_size
    })
}

pub fn used_memory() -> usize {
    interrupts::without_interrupts(|| {
        let state = ALLOC_STATE.lock();
        state.heap.as_ref().map_or(0, |heap| heap.used())
    })
}

pub fn working() -> bool {
    interrupts::without_interrupts(|| {
        let state = ALLOC_STATE.lock();
        state.heap.is_some()
    })
}
