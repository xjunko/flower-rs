use core::alloc::GlobalAlloc;
use core::ptr;

const CHUNK_SIZE: usize = 64 * 1024;
const PAGE_SIZE: usize = 4096;

#[repr(C)]
struct AllocationHeader {
    start: usize,
    size: usize,
}

#[repr(C)]
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

static mut FREE_LIST_HEAD: *mut FreeBlock = ptr::null_mut();

const MIN_FREE_BLOCK_SIZE: usize = core::mem::size_of::<FreeBlock>();
const HEADER_SIZE: usize = core::mem::size_of::<AllocationHeader>();

#[inline]
const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

unsafe fn add_free_block(start: usize, size: usize) {
    if size < MIN_FREE_BLOCK_SIZE {
        return;
    }

    let mut prev: *mut FreeBlock = ptr::null_mut();
    let mut curr = unsafe { FREE_LIST_HEAD };

    while !curr.is_null() && (curr as usize) < start {
        prev = curr;
        curr = unsafe { (*curr).next };
    }

    let new_block = start as *mut FreeBlock;
    unsafe {
        (*new_block).size = size;
        (*new_block).next = curr;
    }

    if prev.is_null() {
        unsafe {
            FREE_LIST_HEAD = new_block;
        }
    } else {
        unsafe {
            (*prev).next = new_block;
        }
    }

    let mut merged = new_block;

    if !prev.is_null() {
        let prev_end = (prev as usize) + unsafe { (*prev).size };
        if prev_end == (merged as usize) {
            unsafe {
                (*prev).size += (*merged).size;
                (*prev).next = (*merged).next;
            }
            merged = prev;
        }
    }

    let next = unsafe { (*merged).next };
    if !next.is_null() {
        let merged_end = (merged as usize) + unsafe { (*merged).size };
        if merged_end == (next as usize) {
            unsafe {
                (*merged).size += (*next).size;
                (*merged).next = (*next).next;
            }
        }
    }
}

unsafe fn map_chunk(min_size: usize) -> bool {
    let request = align_up(min_size.max(CHUNK_SIZE), PAGE_SIZE);
    let mapped = crate::std::mmap(request) as usize;
    if mapped == 0 {
        return false;
    }

    unsafe { add_free_block(mapped, request) };
    true
}

unsafe fn alloc_from_free_list(layout: core::alloc::Layout) -> *mut u8 {
    let align = layout.align().max(core::mem::align_of::<usize>());
    let size = layout.size().max(1);

    let mut prev: *mut FreeBlock = ptr::null_mut();
    let mut curr = unsafe { FREE_LIST_HEAD };

    while !curr.is_null() {
        let block_start = curr as usize;
        let block_size = unsafe { (*curr).size };
        let block_end = block_start + block_size;

        let payload = align_up(block_start + HEADER_SIZE, align);
        let alloc_end = match payload.checked_add(size) {
            Some(v) => v,
            None => return ptr::null_mut(),
        };

        if alloc_end <= block_end {
            let mut alloc_size = alloc_end - block_start;
            let tail_size = block_end - alloc_end;

            if tail_size >= MIN_FREE_BLOCK_SIZE {
                let tail = alloc_end as *mut FreeBlock;
                unsafe {
                    (*tail).size = tail_size;
                    (*tail).next = (*curr).next;
                }

                if prev.is_null() {
                    unsafe { FREE_LIST_HEAD = tail };
                } else {
                    unsafe { (*prev).next = tail };
                }
            } else {
                alloc_size = block_size;
                if prev.is_null() {
                    unsafe { FREE_LIST_HEAD = (*curr).next };
                } else {
                    unsafe { (*prev).next = (*curr).next };
                }
            }

            let header_ptr = (payload - HEADER_SIZE) as *mut AllocationHeader;
            unsafe {
                (*header_ptr).start = block_start;
                (*header_ptr).size = alloc_size;
            }

            return payload as *mut u8;
        }

        prev = curr;
        curr = unsafe { (*curr).next };
    }

    ptr::null_mut()
}

pub struct FlowerLibcAllocator;

unsafe impl GlobalAlloc for FlowerLibcAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let min_chunk = layout
            .size()
            .max(1)
            .saturating_add(HEADER_SIZE)
            .saturating_add(layout.align());

        let ptr = unsafe { alloc_from_free_list(layout) };
        if !ptr.is_null() {
            return ptr;
        }

        if !unsafe { map_chunk(min_chunk) } {
            return ptr::null_mut();
        }

        unsafe { alloc_from_free_list(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if ptr.is_null() {
            return;
        }

        if layout.size() == 0 {
            return;
        }

        let header_ptr =
            (ptr as usize - HEADER_SIZE) as *const AllocationHeader;
        let start = unsafe { (*header_ptr).start };
        let size = unsafe { (*header_ptr).size };

        if start == 0 || size < MIN_FREE_BLOCK_SIZE {
            return;
        }

        unsafe { add_free_block(start, size) };
    }
}
