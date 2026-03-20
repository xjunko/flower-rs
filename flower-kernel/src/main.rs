#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, alloc_error_handler)]
#![allow(dead_code)] // everything is WIP, i dont care
#![allow(clippy::manual_div_ceil)] // i dont trust the .div_ceil implementation 

extern crate alloc;

mod arch;
mod boot;
mod drivers;
mod system;

use alloc::{format, vec};

static HELLO_ELF: &[u8] =
    include_bytes!("../../target/x86_64-unknown-none/release/userspace-hello");

static HELLO_C_ELF: &[u8] =
    include_bytes!("../../target/x86_64-unknown-none/release/hello-c");

fn k_init() {
    system::proc::spawn("one-level", || {
        debug!("hello world from {}", system::proc::name());
    });

    system::proc::spawn("two-level", || {
        for i in 0..5 {
            debug!("hello world from {}: {}", system::proc::name(), i);
        }

        system::proc::spawn("two-level-inside", || {
            for i in 0..5 {
                debug!("hello world from {}: {}", system::proc::name(), i);
            }
        })
    });

    system::proc::spawn("three-level", || {
        for i in 0..5 {
            debug!("hello world from {}: {}", system::proc::name(), i);
        }

        system::proc::spawn("three-level-inner", || {
            debug!("hello world from {}", system::proc::name());
            system::proc::spawn("three-level-inner-inside", || {
                for i in 0..5 {
                    debug!("hello world from {}: {}", system::proc::name(), i);
                }
            })
        })
    });

    // stress test the scheduling
    const NUM_PROCESSES: usize = 100;
    for i in 0..NUM_PROCESSES {
        system::proc::spawn(&format!("stress-{}", i), || {
            for j in 0..5 {
                debug!("hello world from {}: {}", system::proc::name(), j);
            }
        });
    }
}

fn k_timer() {
    loop {
        info!("sleeping for 1 second...");
        system::proc::sleep(1000);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(boot::limine::BASE_REVISION.is_supported());
    // com1 serial logging
    drivers::tty::serial::install();

    // cpu init
    arch::install_cpu_features();
    arch::gdt::install();
    arch::idt::install();

    // memory
    system::mem::pmm::install();
    system::mem::vmm::install();
    system::mem::heap::install().expect("failed to install heap");

    // acpi
    arch::acpi::install();

    // apic
    arch::apic::install();

    // syscall
    arch::syscalls::install();

    // scheduler
    system::proc::install();

    // enable interrupts after APIC and scheduler are ready
    arch::interrupts::enable();

    // past this point, the kernel can now do dynamic allocation
    drivers::tty::flanterm::install();

    // self test, more to be added.
    system::mem::self_test();

    // vfs test
    system::vfs::install();

    // file reading test
    let file =
        system::vfs::open("/init/hello.txt", 0).expect("failed to open file");
    let metadata = file.metadata().expect("failed to get metadata");
    info!("file size: {} bytes", metadata.size);

    let mut buf = vec![0u8; metadata.size as usize];
    let bytes_read = file.read(&mut buf).expect("failed to read file");
    info!("read {} bytes from file", bytes_read);
    info!(
        "file contents: {}",
        core::str::from_utf8(&buf).expect("invalid contents")
    );

    // kernel-process test
    system::proc::spawn("init", k_init);
    // system::proc::spawn("timer", k_timer);

    // // usermode process test
    // system::proc::spawn_elf("hello", HELLO_ELF)
    //     .expect("failed to spawn elf process");

    // system::proc::spawn("test-userspace", || {
    //     for i in 0..100 {
    //         system::proc::spawn_elf(&format!("hello-c-{}", i), HELLO_C_ELF)
    //             .expect("failed to spawn elf process");
    //     }
    // });

    warn!("nothing to do, halting!");
    arch::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    error!("panic: {}", _info);
    arch::halt()
}
