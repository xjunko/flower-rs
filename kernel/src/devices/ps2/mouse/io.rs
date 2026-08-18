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

use x86_64::instructions::port::Port;

use crate::devices::ps2::mouse::CMD_WRITE_AUX;

pub fn wait_write(status_port: &mut Port<u8>) {
    for _ in 0..100_000 {
        if unsafe { status_port.read() } & 0x2 == 0 {
            return;
        }
    }
}

pub fn wait_read(status_port: &mut Port<u8>) {
    for _ in 0..100_000 {
        if unsafe { status_port.read() } & 0x1 != 0 {
            return;
        }
    }
}

pub fn write_command(
    cmd_port: &mut Port<u8>,
    status_port: &mut Port<u8>,
    byte: u8,
) {
    wait_write(status_port);
    unsafe { cmd_port.write(byte) };
}

pub fn write_data(
    data_port: &mut Port<u8>,
    status_port: &mut Port<u8>,
    byte: u8,
) {
    wait_write(status_port);
    unsafe { data_port.write(byte) };
}

pub fn read_data(data_port: &mut Port<u8>, status_port: &mut Port<u8>) -> u8 {
    wait_read(status_port);
    unsafe { data_port.read() }
}

pub fn write_aux(
    cmd_port: &mut Port<u8>,
    data_port: &mut Port<u8>,
    status_port: &mut Port<u8>,
    byte: u8,
) -> u8 {
    write_command(cmd_port, status_port, CMD_WRITE_AUX);
    write_data(data_port, status_port, byte);
    read_data(data_port, status_port)
}
