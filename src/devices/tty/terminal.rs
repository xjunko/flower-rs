use alloc::boxed::Box;
use core::fmt::Write;

use os_terminal::font::BitmapFont;
use os_terminal::{DrawTarget, Terminal};
use spin::Once;
use spinning_top::Spinlock;

use crate::system::vfs::VFSFilelike::File;
use crate::system::vfs::{self, VFSFilelike};

type WrappedTerminal = Terminal<FramebufferTerminal>;
static TERMINAL: Once<Spinlock<WrappedTerminal>> = Once::new();

pub struct FramebufferTerminal {
    info_file: Option<VFSFilelike>,
    draw_file: Option<VFSFilelike>,
}

impl DrawTarget for FramebufferTerminal {
    fn size(&self) -> (usize, usize) {
        if let Some(ref fb_info) = self.info_file {
            match fb_info {
                File(f) => {
                    // format goes like this:
                    // [width: u32, height: u32]
                    let mut buf = [0u8; 8];
                    if let Ok(read_bytes) = f.read(&mut buf)
                        && read_bytes == 8
                    {
                        let width =
                            u32::from_le_bytes(buf[0..4].try_into().unwrap())
                                as usize;
                        let height =
                            u32::from_le_bytes(buf[4..8].try_into().unwrap())
                                as usize;
                        return (width, height);
                    }
                },

                _ => {
                    unreachable!()
                },
            }
        }
        (0, 0)
    }

    #[inline(always)]
    fn draw_pixel(&mut self, x: usize, y: usize, rgb: os_terminal::Rgb) {
        if let Some(ref fb_info) = self.draw_file {
            match fb_info {
                File(f) => {
                    // format goes like this:
                    // [x: u32, y: u32, r: u8, g: u8, b: u8]
                    let mut buf = [0u8; 11];
                    buf[0..4].copy_from_slice(&(x as u32).to_le_bytes());
                    buf[4..8].copy_from_slice(&(y as u32).to_le_bytes());
                    buf[8] = rgb.0;
                    buf[9] = rgb.1;
                    buf[10] = rgb.2;

                    if let Ok(written_bytes) = f.write(buf.as_mut_slice())
                        && written_bytes == 11
                    {
                        return;
                    }
                },
                _ => {
                    unreachable!()
                },
            }
        }
    }
}

impl FramebufferTerminal {
    fn new() -> Option<WrappedTerminal> {
        let mut term = FramebufferTerminal { info_file: None, draw_file: None };

        if let Ok(fb_info) = vfs::open("/dev/fb0/info", 0) {
            term.info_file = Some(fb_info);
        }

        if let Ok(fb_draw) = vfs::open("/dev/fb0/draw", 0) {
            term.draw_file = Some(fb_draw);
        }

        let mut terminal = Terminal::new(term, Box::new(BitmapFont));
        terminal.set_color_scheme(0);

        return Some(terminal);
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
        let mut fixer = CrLfFixer { inner: &mut term };
        fixer.write_fmt(args).unwrap();
    }
}
