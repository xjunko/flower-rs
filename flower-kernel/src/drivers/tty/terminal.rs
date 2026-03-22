use alloc::boxed::Box;

use os_terminal::font::BitmapFont;
use os_terminal::{DrawTarget, Terminal};
use spin::MutexGuard;
use spin::mutex::Mutex;

use crate::boot::limine::FRAMEBUFFER_REQUEST;

pub struct FramebufferTerminal {
    buffer: *mut u8,
    width: usize,
    height: usize,

    bpp: usize,
    pitch: usize,
}

unsafe impl Send for FramebufferTerminal {}
unsafe impl Sync for FramebufferTerminal {}

impl DrawTarget for FramebufferTerminal {
    fn size(&self) -> (usize, usize) { (self.width, self.height) }

    #[inline(always)]
    fn draw_pixel(&mut self, x: usize, y: usize, rgb: os_terminal::Rgb) {
        let offset = y * self.pitch + x * self.bpp / 8;
        unsafe {
            let pixel = self.buffer.add(offset) as *mut u32;
            *pixel =
                (rgb.0 as u32) << 16 | (rgb.1 as u32) << 8 | (rgb.2 as u32);
        }
    }
}

pub static CONTEXT: Mutex<Option<Terminal<FramebufferTerminal>>> =
    Mutex::new(None);

pub fn install() {
    if let Some(framebuffer) = FRAMEBUFFER_REQUEST
        .get_response()
        .expect("no framebuffer")
        .framebuffers()
        .next()
    {
        let term = FramebufferTerminal {
            buffer: framebuffer.addr(),
            width: framebuffer.width() as usize,
            height: framebuffer.height() as usize,
            bpp: framebuffer.bpp() as usize,
            pitch: framebuffer.pitch() as usize,
        };

        let terminal = Terminal::new(term, Box::new(BitmapFont));
        *CONTEXT.lock() = Some(terminal);
    }
}

pub fn get() -> MutexGuard<'static, Option<Terminal<FramebufferTerminal>>> {
    CONTEXT.lock()
}
