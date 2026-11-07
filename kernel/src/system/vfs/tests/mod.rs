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

use alloc::sync::Arc;

use flower_abi::fs::{O_CREAT, O_EXCL, O_RDWR};

use crate::system::posix::errno::KError;
use crate::system::vfs::fds::{FdKind, FdTable};
use crate::system::vfs::shm::SharedMemoryFS;
use crate::system::vfs::{FileSystem, KFile, pipe};

fn test_pipe_roundtrip() {
    let (read, write) = pipe::create(false);

    assert_eq!(write.write(b"hello"), Ok(5));

    let mut buf = [0u8; 5];
    assert_eq!(read.read(&mut buf), Ok(5));
    assert_eq!(&buf, b"hello");
}

fn test_pipe_eof() {
    let (read, write) = pipe::create(false);

    assert_eq!(write.write(b"x"), Ok(1));
    drop(write);

    let mut buf = [0u8; 8];
    assert_eq!(read.read(&mut buf), Ok(1));
    assert_eq!(read.read(&mut buf), Ok(0));
}

fn test_pipe_full() {
    let (read, write) = pipe::create(false);
    let data = [0x5Au8; 1024];
    let mut written = 0;

    loop {
        match write.write(&data) {
            Ok(count) => written += count,
            Err(KError::EAGAIN) => break,
            Err(e) => panic!("unexpected pipe error: {:?}", e),
        }
    }

    assert_eq!(written, 8192);

    let mut buf = [0u8; 16];
    assert_eq!(read.read(&mut buf), Ok(16));
    assert_eq!(write.write(&data), Ok(16));
}

fn test_pipe_fd_inheritance() {
    let (read, write) = pipe::create(false);

    let mut parent = FdTable::new();
    let fd = parent.alloc(FdKind::Pipe(read)).expect("fd table full");
    let child = parent.clone();

    assert_eq!(write.write(b"shared"), Ok(6));

    let kind = child.get(fd).expect("inherited fd missing");
    let FdKind::Pipe(pipe) = kind else {
        panic!("inherited fd is not a pipe");
    };

    let mut buf = [0u8; 6];
    assert_eq!(pipe.read(&mut buf), Ok(6));
    assert_eq!(&buf, b"shared");
}

fn test_shmfs_roundtrip() {
    let fs = SharedMemoryFS::new();

    let KFile::Shmem(shmem) =
        fs.open("/test", O_CREAT | O_RDWR).expect("shm create failed")
    else {
        panic!("shm open did not return a shmem object");
    };

    shmem.resize(4096).expect("shm resize failed");

    let KFile::Shmem(again) =
        fs.open("/test", O_RDWR).expect("shm reopen failed")
    else {
        panic!("shm reopen did not return a shmem object");
    };

    assert!(Arc::ptr_eq(&shmem, &again));
    assert_eq!(shmem.len(), 4096);
    assert!(fs.open("/test", O_CREAT | O_EXCL).is_err());

    fs.unlink("/test").expect("shm unlink failed");
    assert!(fs.open("/test", O_RDWR).is_err());
}

pub fn run() {
    test_pipe_roundtrip();
    test_pipe_eof();
    test_pipe_full();
    test_pipe_fd_inheritance();
    test_shmfs_roundtrip();
}
