#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![allow(dead_code)] // everything is WIP, i dont care
#![allow(clippy::manual_div_ceil)] // i dont trust the .div_ceil implementation

extern crate alloc;

mod acpi;
mod arch;
mod boot;
mod devices;
mod logging;
mod memory;
mod system;
mod user;

fn kernel_init() {
    assert!(boot::limine::BASE_REVISION.is_supported());
    devices::tty::serial::install();
    logging::install();

    arch::x86_64::install_cpu_features();
    arch::x86_64::gdt::install();
    arch::x86_64::idt::install();

    memory::pmm::install();
    memory::vmm::install();
    memory::heap::install().expect("failed to install heap");

    acpi::install();
    arch::x86_64::apic::install();

    devices::ps2::install();
    devices::pci::install();
    devices::gpu::install();

    system::syscalls::install();
    system::proc::install();
    arch::x86_64::interrupts::enable();

    // start other cores too
    system::smp::install();

    // past this point, the kernel can now do dynamic allocation
    system::vfs::install();
    devices::tty::terminal::install();
    memory::self_test();
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    kernel_init();
    system::proc::spawn("userland-entry", user::entry);
    arch::x86_64::halt();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    log::error!("panic: {}", _info);
    arch::x86_64::halt()
}
