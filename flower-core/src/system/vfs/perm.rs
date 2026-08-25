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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shift {
    Owner = 6,
    Group = 3,
    Other = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Credentials {
    pub uid: u32,
    pub gid: u32,
}

impl Credentials {
    pub const ROOT: Self = Self { uid: 0, gid: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    bits: u16,
}

impl Permissions {
    pub fn new(bits: u16) -> Self { Self { bits: bits & 0o777 } }

    pub fn from_unix(perm: usize) -> Self { Self::new(perm as u16) }

    pub fn bits(&self) -> u16 { self.bits }

    fn has(&self, access: Access, shift: Shift) -> bool {
        let bit: u16 = match access {
            Access::Read => 0b100,
            Access::Write => 0b010,
            Access::Execute => 0b001,
        };
        (self.bits & (bit << (shift as u16))) != 0
    }

    pub fn check(
        &self,
        who: Credentials,
        owner: u32,
        group: u32,
        access: Access,
    ) -> bool {
        if who.uid == 0 {
            return true;
        }

        let shift = if who.uid == owner {
            Shift::Owner
        } else if who.gid == group {
            Shift::Group
        } else {
            Shift::Other
        };

        self.has(access, shift)
    }
}
