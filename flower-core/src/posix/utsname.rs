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

use flower_uapi::uapi::utsname::utsname;
use spin::mutex::SpinMutex;

const SYSNAME: &[u8] = b"Flower";
const NODENAME: &[u8] = b"flower";
const RELEASE: &[u8] = env!("CARGO_PKG_VERSION").as_bytes();
const VERSION: &[u8] = b"0.0.0";
const MACHINE: &[u8] = b"x86_64";
const DOMAINNAME: &[u8] = b"(none)";

pub static UTSNAME: SpinMutex<utsname> = SpinMutex::new({
    let mut name = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };

    name.sysname[0..SYSNAME.len()].copy_from_slice(SYSNAME);
    name.nodename[0..NODENAME.len()].copy_from_slice(NODENAME);
    name.release[0..RELEASE.len()].copy_from_slice(RELEASE);
    name.version[0..VERSION.len()].copy_from_slice(VERSION);
    name.machine[0..MACHINE.len()].copy_from_slice(MACHINE);
    name.domainname[0..DOMAINNAME.len()].copy_from_slice(DOMAINNAME);

    name
});
