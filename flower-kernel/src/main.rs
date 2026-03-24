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

mod user;

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(boot::limine::BASE_REVISION.is_supported());
    drivers::tty::serial::install();
    drivers::tty::logging::install();

    arch::install_cpu_features();
    arch::gdt::install();
    arch::idt::install();

    system::mem::pmm::install();
    system::mem::vmm::install();
    system::mem::heap::install().expect("failed to install heap");

    arch::acpi::install();
    arch::apic::install();

    drivers::ps2::install();
    drivers::pci::install();

    system::syscalls::install();
    system::proc::install();
    arch::interrupts::enable();

    // past this point, the kernel can now do dynamic allocation
    system::vfs::install();
    drivers::tty::terminal::install();
    system::mem::self_test();
    system::proc::spawn("userland-entry", user::entry);
    arch::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    log::error!("panic: {}", _info);
    arch::halt()
}
