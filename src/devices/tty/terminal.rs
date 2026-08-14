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

use alloc::boxed::Box;
use core::fmt::Write;

use flower_mono::kapi::framebuffer::fb_info;
use os_terminal::font::BitmapFont;
use os_terminal::{DrawTarget, Terminal};
use spin::Once;
use spinning_top::Spinlock;

use crate::system::vfs::VFSFilelike::File;
use crate::system::vfs::{self};

type WrappedTerminal = Terminal<FramebufferTerminal>;
static TERMINAL: Once<Spinlock<WrappedTerminal>> = Once::new();

unsafe impl Send for FramebufferTerminal {}

pub struct FramebufferTerminal {
    fb_addr: Option<*mut u8>,
    fb_info: Option<fb_info>,
}

impl DrawTarget for FramebufferTerminal {
    fn size(&self) -> (usize, usize) {
        if let Some(info) = self.fb_info {
            (info.width as usize, info.height as usize)
        } else {
            (0, 0)
        }
    }

    #[inline(always)]
    fn draw_pixel(&mut self, x: usize, y: usize, rgb: os_terminal::Rgb) {
        if let Some(fb_addr) = self.fb_addr {
            let offset = (y * self.size().0 + x) * 4;
            unsafe {
                let pixel_ptr = fb_addr.add(offset);
                *pixel_ptr = rgb.2;
                *pixel_ptr.add(1) = rgb.1;
                *pixel_ptr.add(2) = rgb.0;
                *pixel_ptr.add(3) = 0;
            }
        }
    }
}

impl FramebufferTerminal {
    fn new() -> Option<WrappedTerminal> {
        let mut term = FramebufferTerminal { fb_info: None, fb_addr: None };

        if let Ok(fb_info) = vfs::open("/dev/fb0/info", 0) {
            match fb_info {
                File(f) => {
                    let mut buf = [0u8; fb_info::SIZE];

                    if let Ok(read_bytes) = f.read(&mut buf)
                        && read_bytes == buf.len()
                        && let Some(info) = fb_info::from_bytes(&buf)
                    {
                        term.fb_info = Some(info);
                    }
                },

                _ => {
                    unreachable!()
                },
            }
        }

        if let Ok(fb_file) = vfs::open("/dev/fb0", 0) {
            match fb_file {
                File(f) => {
                    let addr = f.mmap(0, 0, 0, 0).ok()?;
                    term.fb_addr = Some(addr);
                },

                _ => {
                    unreachable!()
                },
            }
        }

        let mut terminal = Terminal::new(term, Box::new(BitmapFont));
        terminal.set_color_scheme(0);

        Some(terminal)
    }
}

pub fn install() {
    if let Some(term) = FramebufferTerminal::new() {
        TERMINAL.call_once(|| Spinlock::new(term));
    }
}

pub fn uninstall() { todo!() }

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
