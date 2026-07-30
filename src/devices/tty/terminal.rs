use alloc::boxed::Box;
use core::fmt::Write;

use os_terminal::font::BitmapFont;
use os_terminal::{DrawTarget, Terminal};
use spin::Once;
use spinning_top::Spinlock;

use crate::boot::limine::FRAMEBUFFER_REQUEST;

type WrappedTerminal = Terminal<FramebufferTerminal>;
static TERMINAL: Once<Spinlock<WrappedTerminal>> = Once::new();

unsafe impl Send for FramebufferTerminal {}
unsafe impl Sync for FramebufferTerminal {}

pub struct FramebufferTerminal {
    buffer: *mut u8,
    width: usize,
    height: usize,

    bpp: usize,
    pitch: usize,
}

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

impl FramebufferTerminal {
    fn new() -> Option<WrappedTerminal> {
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

            let mut terminal = Terminal::new(term, Box::new(BitmapFont));
            terminal.set_color_scheme(0);

            return Some(terminal);
        }
        None
    }
}

pub fn install() {
    if let Some(term) = FramebufferTerminal::new() {
        TERMINAL.call_once(|| Spinlock::new(term));
    }
}

fn get() -> &'static Spinlock<WrappedTerminal> {
    TERMINAL.get().expect("terminal not installed")
}

pub fn ready() -> bool { TERMINAL.is_completed() }

pub fn print(args: core::fmt::Arguments<'_>) {
    struct CrLfFixer<'a> {
        inner: &'a mut WrappedTerminal,
    }

    impl core::fmt::Write for CrLfFixer<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for byte in s.bytes() {
                match byte {
                    b'\n' => {
                        self.inner.write_char('\r')?;
                        self.inner.write_char('\n')?;
                    },
                    _ => {
                        self.inner.write_char(byte as char)?;
                    },
                }
            }

            Ok(())
        }
    }

    if let Some(term) = TERMINAL.get() {
        let mut term = term.lock();
        let mut fixer = CrLfFixer { inner: &mut *term };
        fixer.write_fmt(args).unwrap();
    }
}
