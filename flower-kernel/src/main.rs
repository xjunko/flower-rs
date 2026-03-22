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

use alloc::format;

static HELLO_ELF: &[u8] =
    include_bytes!("../../target/x86_64-unknown-none/release/userspace-hello");

fn k_stress() {
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
        info!("timer tick from {}", system::proc::name());
        system::proc::sleep(1000);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(boot::limine::BASE_REVISION.is_supported());
    drivers::tty::serial::install();

    arch::install_cpu_features();
    arch::gdt::install();
    arch::idt::install();

    system::mem::pmm::install();
    system::mem::vmm::install();
    system::mem::heap::install().expect("failed to install heap");

    arch::acpi::install();
    arch::apic::install();

    system::syscalls::install();
    system::proc::install();
    arch::interrupts::enable();

    // past this point, the kernel can now do dynamic allocation
    drivers::tty::flanterm::install();
    system::vfs::install();

    system::mem::self_test();

    // kernel-process test
    system::proc::spawn("timer", k_timer);
    system::proc::spawn("stress", k_stress);

    // user-mode process test
    system::proc::spawn_elf("hello", HELLO_ELF)
        .expect("failed to spawn elf process");

    // user-mode shell test
    // if let Ok(file) = system::vfs::open("/init/shell", 0) {
    //     let metadata = file.metadata().expect("invalid metadata");
    //     let mut buffer = alloc::vec![0u8; metadata.size ];
    //     file.read(&mut buffer).expect("failed to read file");
    //     system::proc::spawn_elf("shell", &buffer)
    //         .expect("failed to spawn shell process");
    // } else {
    //     println!("failed to open file /init/shell");
    // }

    warn!("nothing to do, halting!");
    arch::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    error!("panic: {}", _info);
    arch::halt()
}
