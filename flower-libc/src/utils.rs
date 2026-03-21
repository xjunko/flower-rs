pub struct CStr<'a> {
    ptr: *const u8,
    _marker: core::marker::PhantomData<&'a u8>,
}

impl<'a> CStr<'a> {
    /// Construct from a `&[u8]`. Must be null-terminated.
    pub fn from_bytes_with_nul(bytes: &'a [u8]) -> Option<Self> {
        if bytes.last() == Some(&0) {
            Some(Self {
                ptr: bytes.as_ptr(),
                _marker: core::marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// Get the raw pointer to pass to syscalls.
    pub fn as_ptr(&self) -> *const u8 { self.ptr }
}
