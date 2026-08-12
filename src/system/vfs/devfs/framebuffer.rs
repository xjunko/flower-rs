use alloc::string::ToString;
use core::ffi::c_int;

use crate::devices;
use crate::system::vfs::devfs::{DevFS, DevFile};
use crate::system::vfs::{VFSError, VFSResult};

fn framebuffer_read(_offset: usize, _buf: &mut [u8]) -> usize {
    unimplemented!()
}
fn framebuffer_write(_buf: &[u8]) -> usize { unimplemented!() }

fn framebuffer_mmap(
    _size: usize,
    _prot: c_int,
    _flags: c_int,
    _offset: u64,
) -> VFSResult<*mut u8> {
    if let Some(fb) = devices::gpu::fb::get() {
        // TODO: we should probably point it to a virtual address instead.
        Ok(fb.addr())
    } else {
        Err(VFSError::NotFound)
    }
}

fn framebuffer_info(_offset: usize, buf: &mut [u8]) -> usize {
    // format goes like this:
    // [width: u32, height: u32]
    if buf.len() < 8 {
        return 0;
    }

    if let Some(fb) = devices::gpu::fb::get() {
        let width_bytes = (fb.width as u32).to_le_bytes();
        let height_bytes = (fb.height as u32).to_le_bytes();

        buf[0..4].copy_from_slice(&width_bytes);
        buf[4..8].copy_from_slice(&height_bytes);

        return 8;
    }

    0
}

fn framebuffer_draw_pixel(buf: &[u8]) -> usize {
    // format goes like this:
    // [x: u32, y: u32, r: u8, g: u8, b: u8]

    if buf.len() < 11 {
        return 0;
    }

    let x = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;
    let y = u32::from_le_bytes(buf[4..8].try_into().unwrap()) as usize;
    let r = buf[8];
    let g = buf[9];
    let b = buf[10];

    if let Some(fb) = devices::gpu::fb::get() {
        fb.draw_pixel(x, y, (r, g, b));
    }

    0
}

pub fn install(dev: &mut DevFS) {
    dev.bind(DevFile::new(
        "/fb0".to_string(),
        Some(framebuffer_read),
        Some(framebuffer_write),
        Some(framebuffer_mmap),
    ));

    dev.bind(DevFile::new(
        "/fb0/info".to_string(),
        Some(framebuffer_info),
        None,
        None,
    ));

    dev.bind(DevFile::new(
        "/fb0/draw".to_string(),
        None,
        Some(framebuffer_draw_pixel),
        None,
    ));
}
