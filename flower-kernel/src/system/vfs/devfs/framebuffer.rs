use alloc::string::ToString;
use core::ffi::c_int;

use crate::boot::limine::FRAMEBUFFER_REQUEST;
use crate::system::vfs::devfs::{DevFS, DevFile};

fn framebuffer_read(_offset: usize, _buf: &mut [u8]) -> usize {
    unimplemented!()
}
fn framebuffer_write(_buf: &[u8]) -> usize { unimplemented!() }

fn framebuffer_mmap(_size: usize, _prot: c_int, _flags: c_int) -> *mut u8 {
    // HACK: uhh
    if let Some(fb) = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("no valid framebuffer")
        .framebuffers()
        .next()
    {
        fb.addr()
    } else {
        core::ptr::null_mut()
    }
}

pub fn install(dev: &mut DevFS) {
    dev.bind(DevFile::new(
        "/fb0".to_string(),
        Some(framebuffer_read),
        Some(framebuffer_write),
        Some(framebuffer_mmap),
    ));
}
