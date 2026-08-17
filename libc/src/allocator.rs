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

use crate::sys::kernel;

const DEFAULT_HEAP_SIZE: usize = 1024 * 1024;

struct LibcAllocator;
#[global_allocator]
static ALLOCATOR: LibcAllocator = LibcAllocator;

struct AllocStateInner {
    heap: Option<Heap>,
    heap_size: usize,
}

static ALLOC_STATE: Mutex<AllocStateInner> =
    Mutex::new(AllocStateInner { heap: None, heap_size: 0 });

unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut state = ALLOC_STATE.lock();

        if state.heap.is_none() {
            let base = kernel::mmap_anonymous(DEFAULT_HEAP_SIZE);
            assert!(!base.is_null(), "failed to initialize heap");
            state.heap_size = DEFAULT_HEAP_SIZE;
            state.heap = Some(unsafe { Heap::new(base, DEFAULT_HEAP_SIZE) });
        }

        loop {
            if let Some(heap) = state.heap.as_mut()
                && let Ok(ptr) = heap.allocate_first_fit(layout)
            {
                return ptr.as_ptr();
            }

            if let Some(heap) = state.heap.as_mut() {
                let new_base = kernel::mmap_anonymous(DEFAULT_HEAP_SIZE);
                assert!(!new_base.is_null(), "failed to expand heap");
                unsafe {
                    heap.extend(DEFAULT_HEAP_SIZE);
                }
                state.heap_size += DEFAULT_HEAP_SIZE;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut state = ALLOC_STATE.lock();
        if let Some(heap) = state.heap.as_mut() {
            unsafe {
                heap.deallocate(core::ptr::NonNull::new(ptr).unwrap(), layout)
            }
        }
    }
}

pub fn install() {
    // noop
}

pub fn uninstall() {
    let state = ALLOC_STATE.lock();
    if let Some(heap) = state.heap.as_ref() {
        let heap_start = heap.bottom();
        let heap_size = state.heap_size;
        kernel::munmap(heap_start, heap_size);
    }
}
