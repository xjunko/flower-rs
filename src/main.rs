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
#![feature(const_index)]
#![feature(const_trait_impl)]
#![feature(abi_x86_interrupt)]
#![allow(dead_code)]
#![allow(clippy::manual_div_ceil)]

extern crate alloc;

mod acpi;
mod arch;
mod boot;
mod devices;
mod logging;
mod memory;
mod posix;
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
    system::vfs2::install();
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
