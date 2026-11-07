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

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use spin::Once;

// alignment
#[inline]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

#[inline]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        if let Some(aligned) = (addr | align_mask).checked_add(1) {
            aligned
        } else {
            panic!("attempt to add with overflow")
        }
    }
}

// hhdm alignment for internal usage
pub(crate) static HHDM: Once<KVirtAddr> = Once::new();

// addr
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct KVirtAddr(pub u64);

impl KVirtAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }

    pub const fn as_u64(&self) -> u64 { self.0 }

    pub const fn as_ptr<T>(self) -> *const T { self.as_u64() as *const T }

    pub const fn as_mut_ptr<T>(self) -> *mut T { self.as_ptr::<T>() as *mut T }
}

impl fmt::Debug for KVirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VirtAddr").field(&format_args!("{:#x}", self.0)).finish()
    }
}

impl fmt::Binary for KVirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for KVirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for KVirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for KVirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for KVirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for KVirtAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        KVirtAddr::new(
            self.0.checked_add(rhs).expect("attempt to add with overflow"),
        )
    }
}

impl AddAssign<u64> for KVirtAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) { *self = *self + rhs; }
}

impl Sub<u64> for KVirtAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        KVirtAddr::new(
            self.0.checked_sub(rhs).expect("attempt to subtract with overflow"),
        )
    }
}

impl SubAssign<u64> for KVirtAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) { *self = *self - rhs; }
}

impl Sub<KVirtAddr> for KVirtAddr {
    type Output = u64;

    #[inline]
    fn sub(self, rhs: KVirtAddr) -> Self::Output {
        self.as_u64()
            .checked_sub(rhs.as_u64())
            .expect("attempt to subtract with overflow")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct KPhysAddr(pub u64);

impl KPhysAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }

    pub const fn as_u64(&self) -> u64 { self.0 }

    pub const fn as_ptr<T>(self) -> *const T { self.as_u64() as *const T }

    pub const fn as_mut_ptr<T>(self) -> *mut T { self.as_ptr::<T>() as *mut T }

    pub fn as_hhdm<T>(self) -> *mut T {
        (*HHDM.get().expect("no hhdm installed") + self.0).as_mut_ptr()
    }
}

impl fmt::Debug for KPhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PhysAddr").field(&format_args!("{:#x}", self.0)).finish()
    }
}

impl fmt::Binary for KPhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for KPhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for KPhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for KPhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for KPhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for KPhysAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        KPhysAddr::new(self.0.checked_add(rhs).unwrap())
    }
}

impl AddAssign<u64> for KPhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) { *self = *self + rhs; }
}

impl Sub<u64> for KPhysAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        KPhysAddr::new(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<u64> for KPhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) { *self = *self - rhs; }
}

impl Sub<KPhysAddr> for KPhysAddr {
    type Output = u64;

    #[inline]
    fn sub(self, rhs: KPhysAddr) -> Self::Output {
        self.as_u64().checked_sub(rhs.as_u64()).unwrap()
    }
}
