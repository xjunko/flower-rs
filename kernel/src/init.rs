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
// common feature
#![feature(const_index)]
#![feature(const_trait_impl)]
// arch feature
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
// clippy
#![allow(dead_code)]

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
    boot::install();

    system::mem::install();

    acpi::install();
    arch::timers::install();

    drivers::ps2::install();
    drivers::pci::install();
    drivers::fb::install();

    arch::syscalls::install();
    system::proc::install();
    arch::interrupts::enable();

    // past this point, the vmm _should_ work...
    system::vfs::install();
    drivers::tty::terminal::install();
    system::mem::self_test();
}

fn kcrash(info: &core::panic::PanicInfo) {
    drivers::tty::terminal::set_crash_color_scheme();

    log::error!("!!!! KERNEL PANIC !!!!");
    log::error!("reason: {}", info.message());
    if let Some(location) = info.location() {
        log::error!(
            "location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        log::error!("location: unknown");
    }
    log::error!("!!!! KERNEL PANIC !!!!");

    drivers::tty::terminal::uninstall();
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(thread) = system::proc::current() {
        let thread_id = thread.lock().id;

        // if we're on tid 0 just bail out.
        if thread_id == 0 {
            kcrash(info);
        } else {
            log::error!("panicked on thread: {:?}", thread_id);
            log::error!("reason: {}", info.message());

            if let Some(location) = info.location() {
                log::error!(
                    "location: {}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                );
            } else {
                log::error!("location: unknown");
            }
        }
    }

    arch::cpu::halt()
}
