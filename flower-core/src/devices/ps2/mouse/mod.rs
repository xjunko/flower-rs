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

pub mod handler;
mod io;
pub mod publisher;

use x86_64::instructions::port::Port;

use crate::devices::ps2::mouse::io::{
    read_data, write_aux, write_command, write_data,
};

pub const PS2_DATA: u16 = 0x60;
pub const PS2_STATUS: u16 = 0x64;
pub const PS2_CMD: u16 = 0x64;

pub const CMD_ENABLE_AUX: u8 = 0xA8;
pub const CMD_READ_CONFIG: u8 = 0x20;
pub const CMD_WRITE_CONFIG: u8 = 0x60;
pub const CMD_WRITE_AUX: u8 = 0xD4;

pub const MOUSE_SET_DEFAULTS: u8 = 0xF6;
pub const MOUSE_ENABLE_REPORTING: u8 = 0xF4;

pub const MOUSE_ACK: u8 = 0xFA;

pub fn install() {
    let mut cmd_port: Port<u8> = Port::new(PS2_CMD);
    let mut status_port: Port<u8> = Port::new(PS2_STATUS);
    let mut data_port: Port<u8> = Port::new(PS2_DATA);

    // enable
    write_command(&mut cmd_port, &mut status_port, CMD_ENABLE_AUX);

    // read config
    write_command(&mut cmd_port, &mut status_port, CMD_READ_CONFIG);
    let mut config = read_data(&mut data_port, &mut status_port);
    config |= 0b0000_0010; // bit 1: enable interrupt (irq 12)
    config &= !0b0010_0000; // bit 5: enable aux clock

    write_command(&mut cmd_port, &mut status_port, CMD_WRITE_CONFIG);
    write_data(&mut data_port, &mut status_port, config);

    // set aux config to default
    let ack = write_aux(
        &mut cmd_port,
        &mut data_port,
        &mut status_port,
        MOUSE_SET_DEFAULTS,
    );
    if ack != MOUSE_ACK {
        log::warn!("ps2::mouse: unexpected ack {:#x} on set-defaults", ack);
    }

    // enable aux reporting
    let ack = write_aux(
        &mut cmd_port,
        &mut data_port,
        &mut status_port,
        MOUSE_ENABLE_REPORTING,
    );
    if ack != MOUSE_ACK {
        log::warn!("ps2::mouse: unexpected ack {:#x} on enable-reporting", ack);
    }

    log::debug!("ps2::mouse installed!");
}
