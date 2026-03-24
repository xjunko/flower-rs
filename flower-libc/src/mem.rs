use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use good_memory_allocator::SpinLockedAllocator;

use crate::std;

// ought to be enough...
const DEFAULT_HEAP_SIZE: usize = 2 * 1024 * 1024;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static HEAP_START: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

pub fn install() {
    if INSTALLED.load(Ordering::Acquire) {
        return;
    }

    let heap_start = std::mmap((-1i64) as u64, DEFAULT_HEAP_SIZE);
    assert!(!heap_start.is_null(), "failed to initialize userspace heap");

    unsafe {
        ALLOCATOR.init(heap_start as usize, DEFAULT_HEAP_SIZE);
    }

    HEAP_START.store(heap_start as usize, Ordering::Release);
    INSTALLED.store(true, Ordering::Release);
}

pub fn uninstall() {
    if !INSTALLED.swap(false, Ordering::AcqRel) {
        return;
    }

    let heap_start = HEAP_START.swap(0, Ordering::AcqRel);
    if heap_start != 0 {
        let _ = std::munmap(heap_start as *mut u8, DEFAULT_HEAP_SIZE);
    }
}
