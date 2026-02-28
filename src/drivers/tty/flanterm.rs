use core::{fmt::Write, ptr::null_mut};

use flanterm::sys::{flanterm_context, flanterm_fb_init, flanterm_write};
use spin::Mutex;

use crate::{boot::limine::FRAMEBUFFER_REQUEST, debug, error, info};

pub static CONTEXT: Mutex<Option<FlantermContext>> = Mutex::new(None);

unsafe impl Send for FlantermContext {}
unsafe impl Sync for FlantermContext {}
pub struct FlantermContext(*mut flanterm_context, bool);

impl FlantermContext {
    pub fn write(&mut self, byte: u8) {
        if !self.1 {
            return;
        }

        unsafe { flanterm_write(self.0, &byte as *const u8 as *const i8, 1) };
    }
}

impl Write for FlantermContext {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write(c as u8);
        }
        Ok(())
    }
}

pub fn install() {
    for framebuffer in FRAMEBUFFER_REQUEST
        .get_response()
        .expect("no framebuffer")
        .framebuffers()
    {
        debug!(
            "framebuffer: addr={:#x}, size={}x{}, pitch={}",
            framebuffer.addr() as usize,
            framebuffer.width(),
            framebuffer.height(),
            framebuffer.pitch(),
        );

        unsafe {
            let raw = flanterm_fb_init(
                None,
                None,
                framebuffer.addr().cast(),
                framebuffer.width() as _,
                framebuffer.height() as _,
                framebuffer.pitch() as _,
                framebuffer.red_mask_size(),
                framebuffer.red_mask_shift(),
                framebuffer.green_mask_size(),
                framebuffer.green_mask_shift(),
                framebuffer.blue_mask_size(),
                framebuffer.blue_mask_shift(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0,
                0,
                1,
                0,
                0,
                10,
            );

            if raw.is_null() {
                error!("failed to initialize flanterm");
            } else {
                *CONTEXT.lock() = Some(FlantermContext(raw, true));
                info!("flanterm initialized successfully");
                break;
            }
        }
    }
}
