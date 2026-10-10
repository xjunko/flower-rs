use alloc::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct VirtAddr(u64);

impl VirtAddr {
    #[inline]
    pub fn new(addr: u64) -> Self {
        match Self::try_new(addr) {
            Ok(v) => v,
            Err(e) => panic!("invalid virtual address: {}", e),
        }
    }

    #[inline]
    pub const fn try_new(addr: u64) -> Result<Self, &'static str> {
        #[cfg(target_arch = "x86_64")]
        {
            let v = Self(((addr << 16) as i64 >> 16) as u64);

            if v.0 == addr {
                Ok(v)
            } else {
                Err("bits 48-64 must be sign extended")
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        return Ok(Self(addr));
    }

    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }

    #[inline]
    pub const fn zero() -> Self { Self(0) }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_ptr<T>(&self) -> *const T { self.0 as *const T }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_mut_ptr<T>(&self) -> *mut T { self.0 as *mut T }

    #[inline]
    pub const fn is_null(&self) -> bool { self.0 == 0 }
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct PhysAddr(u64);

impl PhysAddr {
    #[inline]
    pub fn new(addr: u64) -> Self { Self(addr) }

    #[inline]
    pub const fn try_new(addr: u64) -> Result<Self, &'static str> {
        #[cfg(target_arch = "x86_64")]
        {
            let v = Self(addr % (1 << 52));

            if v.0 == addr { Ok(v) } else { Err("bits 52-64 must not be set") }
        }

        #[cfg(not(target_arch = "x86_64"))]
        return Ok(Self(addr));
    }

    #[inline]
    pub const fn as_u64(&self) -> u64 { self.0 }

    #[inline]
    pub const fn zero() -> Self { Self(0) }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_ptr<T>(&self) -> *const T { self.0 as *const T }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_mut_ptr<T>(&self) -> *mut T { self.0 as *mut T }

    #[inline]
    pub const fn is_null(&self) -> bool { self.0 == 0 }
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
