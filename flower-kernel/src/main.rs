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
#![feature(abi_x86_interrupt)]
#![allow(dead_code)] // everything is WIP, i dont care
#![allow(clippy::manual_div_ceil)] // i dont trust the .div_ceil implementation

extern crate alloc;

mod acpi;
mod arch;
mod boot;
mod drivers;
mod logging;
mod system;

mod user;

#[unsafe(no_mangle)]
unsafe extern "C" fn __entry() -> ! {
    kmain();
    system::proc::spawn("userland-entry", user::entry);
    arch::cpu::halt();
}

fn kmain() {
    assert!(boot::limine::BASE_REVISION.is_supported());
    drivers::tty::serial::install();
    logging::install();

    arch::cpu::bsp_install();

    system::mem::pmm::install();
    system::mem::vmm::install();
    system::mem::heap::install().expect("failed to install heap");

    acpi::install();
    arch::timers::install();

    drivers::ps2::install();
    drivers::pci::install();

    arch::syscalls::install();
    system::proc::install();
    arch::interrupts::enable();

    // past this point, the kernel can now do dynamic allocation
    system::vfs::install();
    drivers::tty::terminal::install();
    system::mem::self_test();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    log::error!("panic: {}", _info);
    arch::cpu::halt()
}
