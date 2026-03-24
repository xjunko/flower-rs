#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec;
use core::cmp::min;

use flower_libc::file::File;
use flower_libc::{println, std};

const FB_WIDTH: usize = 1280;
const FB_HEIGHT: usize = 720;
const FB_PITCH: usize = FB_WIDTH * 4;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    flower_libc::_init();

    let argc = flower_libc::auxv::argc();

    if argc < 2 {
        println!("usage: png <filename>");
        std::exit(0);
    }

    let file_path = match flower_libc::auxv::argv(1) {
        Some(path) => path,
        None => {
            println!("failed to get filename argument");
            std::exit(1);
        },
    };

    std::exit(cat(file_path) as u64);
}

pub fn cat(filename: &str) -> i32 {
    if filename.is_empty() {
        println!("usage: png <filename>");
        return 1;
    }

    // parse a png
    if let Ok(file) = File::open(filename.to_string()) {
        let mut buf = vec![0u8; 1024 * 1024]; // 1mb for now...
        file.read(&mut buf).expect("failed to read full image");
        file.close().expect("failed to close file");

        let (header, image_data) = png_decoder::decode(&buf).unwrap();
        println!("PNG header: {:?}", header);
        println!("Image data length: {}", image_data.len());

        // get the framebuffer
        let fb_fd = std::open(b"/dev/fb0", 0, 0);
        if fb_fd < 0 {
            println!("failed to open /dev/fb0");
            return -1;
        }

        let fb_addr = std::mmap(fb_fd as u64, FB_PITCH * FB_HEIGHT);
        if fb_addr.is_null() {
            println!("failed to mmap /dev/fb0");
            return -1;
        }

        draw_rgba_to_framebuffer(
            fb_addr,
            FB_WIDTH,
            FB_HEIGHT,
            FB_PITCH,
            header.width as usize,
            header.height as usize,
            &image_data,
        );

        std::munmap(fb_addr, FB_PITCH * FB_HEIGHT);
        std::close(fb_fd as u64);
    } else {
        println!("failed to open {}", filename);
        return -1;
    }

    0
}

fn blend_channel(src: u8, dst: u8, alpha: u8) -> u8 {
    let alpha = alpha as u16;
    let inv_alpha = 255u16.saturating_sub(alpha);
    (((src as u16 * alpha) + (dst as u16 * inv_alpha)) / 255) as u8
}

fn draw_rgba_to_framebuffer(
    fb_addr: *mut u8,
    fb_width: usize,
    fb_height: usize,
    fb_pitch: usize,
    img_width: usize,
    img_height: usize,
    image_data: &[[u8; 4]],
) {
    let draw_width = min(img_width, fb_width);
    let draw_height = min(img_height, fb_height);

    for y in 0..draw_height {
        let row_start = y * img_width;
        for x in 0..draw_width {
            let [src_r, src_g, src_b, src_a] = image_data[row_start + x];
            let pixel_offset = y * fb_pitch + x * 4;

            unsafe {
                let pixel = fb_addr.add(pixel_offset) as *mut u32;
                let dst = *pixel;

                let dst_r = ((dst >> 16) & 0xFF) as u8;
                let dst_g = ((dst >> 8) & 0xFF) as u8;
                let dst_b = (dst & 0xFF) as u8;

                let out_r = if src_a == 255 {
                    src_r
                } else {
                    blend_channel(src_r, dst_r, src_a)
                };
                let out_g = if src_a == 255 {
                    src_g
                } else {
                    blend_channel(src_g, dst_g, src_a)
                };
                let out_b = if src_a == 255 {
                    src_b
                } else {
                    blend_channel(src_b, dst_b, src_a)
                };

                *pixel =
                    (out_r as u32) << 16 | (out_g as u32) << 8 | out_b as u32;
            }
        }
    }
}
