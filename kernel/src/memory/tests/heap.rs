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

use alloc::boxed::Box;
use alloc::vec::Vec;

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

pub fn test_heap_large() { let _ = Vec::<u8>::with_capacity(512 * 1024); }

pub fn test_heap_oom() {
    // HACK: this works but it crashes the kernel
    // and we have no way of recovering from it, since it's done in kernel space
    // let mut vec: Vec<u8> = Vec::new();
    // vec.try_reserve(128 * 1024 * 1024).expect("oom test failed!");
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
