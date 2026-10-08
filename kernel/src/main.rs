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

#![no_std]
#![no_main]
#![allow(dead_code)]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]

extern crate alloc;

use alloc::vec;

use crate::arch::{Arch, Processor};
use crate::vfs::file::OpenFlags;
use crate::vfs::traits::File;

mod acpi;
mod arch;
mod boot;
mod devices;
mod logging;
mod memory;
mod vfs;

fn _kernel_init() {
    devices::tty::serial::install();
    logging::install();

    Processor::bsp_install();

    memory::pmm::install();
    memory::vmm::install();
    memory::heap::install().expect("failed to install heap");

    acpi::install();
    Processor::timer_install();
    Processor::interrupts_enable();

    vfs::install().expect("failed to install vfs");

    // test vfs
    let file = vfs::open("/tmp/test.txt", OpenFlags::CREATE | OpenFlags::WRITE)
        .expect("failed to open file");

    let data = b"Hello, world!";
    file.write(data).expect("failed to write to file");
    drop(file); // close the file

    // read it back
    let another_file = vfs::open("/tmp/test.txt", OpenFlags::READ)
        .expect("failed to open file for reading");
    let metadata = another_file.metadata().expect("failed to get metadata");
    let mut buffer = vec![0u8; metadata.size as usize];
    let bytes_read =
        another_file.read(&mut buffer).expect("failed to read from file");
    assert_eq!(bytes_read, metadata.size as usize);
    assert_eq!(&buffer, data);

    log::info!("vfs test passed: read back {} bytes: {:?}", bytes_read, buffer);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn __kernel_init() -> ! {
    assert!(boot::limine::BASE_REVISION.is_supported());
    _kernel_init();
    Processor::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    log::error!("panic: {}", _info);
    Processor::halt();
}
