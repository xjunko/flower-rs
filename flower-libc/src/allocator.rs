use core::alloc::GlobalAlloc;

use linked_list_allocator::Heap;
use spin::Mutex;

use crate::std;

const MAP_MEMORY: u64 = u64::MAX;
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
            let base = std::mmap(MAP_MEMORY, DEFAULT_HEAP_SIZE);
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
                let new_base = std::mmap(MAP_MEMORY, DEFAULT_HEAP_SIZE);
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
        std::munmap(heap_start, heap_size);
    }
}
