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

/*
* The MIT License (MIT)
*
* Copyright (c) 2018 Philipp Oppermann
* Copyright (c) 2015 Gerd Zellweger
* Copyright (c) 2015 The libcpu Developers
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in
* all copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
* THE SOFTWARE.
*/

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::arch::{align_down, align_up};

const ADDRESS_SPACE_SIZE: u64 = 0x1_0000_0000_0000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

pub struct VirtAddrNotValid(pub u64);

impl core::fmt::Debug for VirtAddrNotValid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl VirtAddr {
    #[inline]
    pub const fn new(addr: u64) -> VirtAddr {
        match Self::try_new(addr) {
            Ok(v) => v,
            Err(_) => {
                panic!("virtual address must be sign extended in bits 48 to 64")
            },
        }
    }

    #[inline]
    pub const fn try_new(addr: u64) -> Result<VirtAddr, VirtAddrNotValid> {
        let v = Self::new_truncate(addr);
        if v.0 == addr { Ok(v) } else { Err(VirtAddrNotValid(addr)) }
    }

    #[inline]
    pub const fn new_truncate(addr: u64) -> VirtAddr {
        VirtAddr(((addr << 16) as i64 >> 16) as u64)
    }

    #[inline]
    pub const unsafe fn new_unsafe(addr: u64) -> VirtAddr { VirtAddr(addr) }

    #[inline]
    pub const fn zero() -> VirtAddr { VirtAddr(0) }

    #[inline]
    pub const fn as_u64(self) -> u64 { self.0 }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Self::new(ptr as *const () as u64)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_ptr<T>(self) -> *const T { self.as_u64() as *const T }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_mut_ptr<T>(self) -> *mut T { self.as_ptr::<T>() as *mut T }

    #[inline]
    pub const fn is_null(self) -> bool { self.0 == 0 }

    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where U: Into<u64> {
        VirtAddr::new_truncate(align_up(self.0, align.into()))
    }

    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where U: Into<u64> {
        self.align_down_u64(align.into())
    }

    #[inline]
    pub(crate) const fn align_down_u64(self, align: u64) -> Self {
        VirtAddr::new_truncate(align_down(self.0, align))
    }

    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where U: Into<u64> {
        self.is_aligned_u64(align.into())
    }

    #[inline]
    pub(crate) const fn is_aligned_u64(self, align: u64) -> bool {
        self.align_down_u64(align).as_u64() == self.as_u64()
    }
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
        VirtAddr::try_new(
            self.0.checked_add(rhs).expect("attempt to add with overflow"),
        )
        .expect("attempt to add resulted in non-canonical virtual address")
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
        VirtAddr::try_new(
            self.0.checked_sub(rhs).expect("attempt to subtract with overflow"),
        )
        .expect("attempt to subtract resulted in non-canonical virtual address")
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

pub struct PhysAddrNotValid(pub u64);

impl core::fmt::Debug for PhysAddrNotValid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PhysAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl PhysAddr {
    #[inline]
    pub const fn new(addr: u64) -> Self {
        match Self::try_new(addr) {
            Ok(p) => p,
            Err(_) => panic!(
                "physical addresses must not have any bits in the range 52 to 64 set"
            ),
        }
    }

    #[inline]
    pub const fn new_truncate(addr: u64) -> PhysAddr {
        PhysAddr(addr % (1 << 52))
    }

    #[inline]
    pub const unsafe fn new_unsafe(addr: u64) -> PhysAddr { PhysAddr(addr) }

    #[inline]
    pub const fn try_new(addr: u64) -> Result<Self, PhysAddrNotValid> {
        let p = Self::new_truncate(addr);
        if p.0 == addr { Ok(p) } else { Err(PhysAddrNotValid(addr)) }
    }

    #[inline]
    pub const fn zero() -> PhysAddr { PhysAddr(0) }

    #[inline]
    pub const fn as_u64(self) -> u64 { self.0 }

    #[inline]
    pub const fn is_null(self) -> bool { self.0 == 0 }

    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where U: Into<u64> {
        PhysAddr::new(align_up(self.0, align.into()))
    }

    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where U: Into<u64> {
        self.align_down_u64(align.into())
    }

    #[inline]
    pub(crate) const fn align_down_u64(self, align: u64) -> Self {
        PhysAddr(align_down(self.0, align))
    }

    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where U: Into<u64> {
        self.is_aligned_u64(align.into())
    }

    #[inline]
    pub(crate) const fn is_aligned_u64(self, align: u64) -> bool {
        self.align_down_u64(align).as_u64() == self.as_u64()
    }
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
