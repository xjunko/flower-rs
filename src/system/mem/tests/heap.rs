use alloc::{boxed::Box, vec::Vec};

pub fn test_heap_basic() {
    let x = Box::new(100);
    assert_eq!(*x, 100);
}

pub fn test_heap_vec() {
    let mut v = Vec::new();

    for i in 0..10000 {
        v.push(i);
    }

    for (i, item) in v.iter().enumerate().take(10000) {
        assert_eq!(*item, i);
    }
}

pub fn test_heap_fragmentation() {
    let mut boxes = Vec::new();

    for i in 0..1000 {
        boxes.push(Box::new(i));
    }

    drop(boxes);
}

pub fn test_heap_large() {
    let _ = Vec::<u8>::with_capacity(512 * 1024);
}

pub fn test_heap_oom() {
    // HACK: this works but it crashes the kernel
    // and we have no way of recovering from it, since it's done in kernel space
    // let _ = Vec::<u8>::with_capacity(2 * 1024 * 1024);
}

pub fn test_heap_stress() {
    let mut vecs = Vec::new();

    for i in 0..500 {
        let mut v = Vec::with_capacity(128);
        for j in 0..128 {
            v.push(i * j);
        }
        vecs.push(v);
    }

    drop(vecs);
}
