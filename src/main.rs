#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![allow(dead_code)]
#![allow(clippy::manual_div_ceil)]

extern crate alloc;

mod arch;
mod boot;
mod drivers;
mod system;

use alloc::vec;

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(boot::limine::BASE_REVISION.is_supported());
    // com1 serial logging
    drivers::tty::serial::install();

    // cpu init
    arch::install();

    // memory
    system::mem::pmm::install();
    system::mem::vmm::install();
    system::mem::heap::install().expect("failed to install heap");

    // past this point, the kernel can now do dynamic allocation
    drivers::tty::flanterm::install();

    // self test, more to be added.
    system::mem::self_test();

    // vfs test
    system::vfs::install();

    // file reading test
    let file = system::vfs::open("/init/hello.txt", 0).expect("failed to open file");
    let metadata = file.metadata().expect("failed to get metadata");
    info!("file size: {} bytes", metadata.size);

    let mut buf = vec![0u8; metadata.size as usize];
    let bytes_read = file.read(&mut buf).expect("failed to read file");
    info!("read {} bytes from file", bytes_read);
    info!(
        "file contents: {}",
        core::str::from_utf8(&buf).expect("invalid contents")
    );

    // test breakpoint
    x86_64::instructions::interrupts::int3();

    warn!("nothing to do, halting!");
    arch::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    error!("panic: {}", _info);
    arch::halt()
}
