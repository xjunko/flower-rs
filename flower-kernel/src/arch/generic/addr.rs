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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }

    pub const fn as_u64(&self) -> u64 { self.0 }

    pub const fn as_ptr<T>(self) -> *const T { self.as_u64() as *const T }

    pub const fn as_mut_ptr<T>(self) -> *mut T { self.as_ptr::<T>() as *mut T }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VirtAddr").field(&format_args!("{:#x}", self.0)).finish()
    }
}

impl fmt::Binary for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::new(
            self.0.checked_add(rhs).expect("attempt to add with overflow"),
        )
    }
}

impl AddAssign<u64> for VirtAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) { *self = *self + rhs; }
}

impl Sub<u64> for VirtAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::new(
            self.0.checked_sub(rhs).expect("attempt to subtract with overflow"),
        )
    }
}

impl SubAssign<u64> for VirtAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) { *self = *self - rhs; }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = u64;

    #[inline]
    fn sub(self, rhs: VirtAddr) -> Self::Output {
        self.as_u64()
            .checked_sub(rhs.as_u64())
            .expect("attempt to subtract with overflow")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self { Self(addr) }

    pub const fn as_u64(&self) -> u64 { self.0 }

    pub const fn as_ptr<T>(self) -> *const T { self.as_u64() as *const T }

    pub const fn as_mut_ptr<T>(self) -> *mut T { self.as_ptr::<T>() as *mut T }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PhysAddr").field(&format_args!("{:#x}", self.0)).finish()
    }
}

impl fmt::Binary for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_add(rhs).unwrap())
    }
}

impl AddAssign<u64> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) { *self = *self + rhs; }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<u64> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) { *self = *self - rhs; }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = u64;

    #[inline]
    fn sub(self, rhs: PhysAddr) -> Self::Output {
        self.as_u64().checked_sub(rhs.as_u64()).unwrap()
    }
}
