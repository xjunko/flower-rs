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

use pci_types::ConfigRegionAccess;
use x86_64::instructions::port::Port;

const CMD_PORT: u16 = 0xCF8;
const DATA_PORT: u16 = 0xCFC;

pub struct PciIO;

impl ConfigRegionAccess for PciIO {
    unsafe fn read(&self, address: acpi::PciAddress, offset: u16) -> u32 {
        let addr: u32 = (1 << 31)
            | ((address.bus() as u32) << 16)
            | ((address.device() as u32) << 11)
            | ((address.function() as u32) << 8)
            | ((offset as u32) & 0xFC);

        let mut cmd = Port::<u32>::new(CMD_PORT);
        let mut data = Port::<u32>::new(DATA_PORT);

        unsafe {
            cmd.write(addr);
            data.read()
        }
    }

    unsafe fn write(&self, address: acpi::PciAddress, offset: u16, value: u32) {
        let addr: u32 = (1 << 31)
            | ((address.bus() as u32) << 16)
            | ((address.device() as u32) << 11)
            | ((address.function() as u32) << 8)
            | ((offset as u32) & 0xFC);

        let mut cmd = Port::<u32>::new(CMD_PORT);
        let mut data = Port::<u32>::new(DATA_PORT);

        unsafe {
            cmd.write(addr);
            data.write(value)
        }
    }
}
