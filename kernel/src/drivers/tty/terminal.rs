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
use core::fmt::{self, Write};

use os_terminal::font::BitmapFont;
use os_terminal::{DrawTarget, Palette, Terminal};
use spin::Mutex;
use spin::mutex::MutexGuard;

use super::CrlfWriter;
use crate::drivers;
use crate::drivers::fb::Framebuffer;

struct FramebufferTerminal {
    fb: Framebuffer,
}

impl DrawTarget for FramebufferTerminal {
    fn size(&self) -> (usize, usize) {
        (self.fb.width as usize, self.fb.height as usize)
    }

    #[inline(always)]
    fn draw_pixel(&mut self, x: usize, y: usize, rgb: os_terminal::Rgb) {
        self.fb.draw(x, y, (rgb.0, rgb.1, rgb.2));
    }
}

type Context = Option<Terminal<FramebufferTerminal>>;

static CONTEXT: Mutex<Context> = Mutex::new(None);

pub fn install() {
    if let Some(fb) = drivers::fb::get() {
        let term = FramebufferTerminal { fb };

        let mut terminal = Terminal::new(term, Box::new(BitmapFont));
        terminal.set_color_scheme(7);
        *CONTEXT.lock() = Some(terminal);
    }
}

pub fn uninstall() { *CONTEXT.lock() = None; }

fn write_to(context: &mut MutexGuard<'static, Context>, args: fmt::Arguments) {
    if let Some(term) = context.as_mut() {
        let mut writer = CrlfWriter::new(term);
        let _ = writer.write_fmt(args);
    }
}

pub fn write_fmt(args: fmt::Arguments) {
    let mut context = CONTEXT.lock();
    write_to(&mut context, args);
}

pub fn try_write_fmt(args: fmt::Arguments) {
    if let Some(mut context) = CONTEXT.try_lock() {
        write_to(&mut context, args);
    }
}

pub fn set_crash_color_scheme() {
    const CRASH_PALETTE: Palette = Palette {
        foreground: (0xff, 0xe6, 0xe6),
        background: (0x33, 0x00, 0x00),
        ansi_colors: [
            (0x33, 0x00, 0x00),
            (0xff, 0x00, 0x00),
            (0xcc, 0x33, 0x33),
            (0xff, 0x66, 0x66),
            (0x99, 0x00, 0x00),
            (0xff, 0x33, 0x33),
            (0xcc, 0x00, 0x00),
            (0xff, 0xe6, 0xe6),
            (0x66, 0x00, 0x00),
            (0xff, 0x33, 0x33),
            (0xff, 0x66, 0x66),
            (0xff, 0x99, 0x99),
            (0xcc, 0x33, 0x33),
            (0xff, 0x4d, 0x4d),
            (0xff, 0x1a, 0x1a),
            (0xff, 0xff, 0xff),
        ],
    };

    let mut context = CONTEXT.lock();
    if let Some(term) = context.as_mut() {
        term.set_custom_color_scheme(&CRASH_PALETTE);
    }
}
