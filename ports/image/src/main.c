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

#include <fcntl.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#define STB_IMAGE_IMPLEMENTATION
#include "thirdparty/stb_image.h"

#define FB_WIDTH 1280
#define FB_HEIGHT 720
#define FB_PITCH (FB_WIDTH * 4)
#define FB_SIZE (FB_PITCH * FB_HEIGHT)

static uint8_t blend_channel(uint8_t src, uint8_t dst, uint8_t alpha) {
  uint16_t a = alpha;
  uint16_t inv_alpha = 255 - a;

  return (uint8_t)(((uint16_t)src * a + (uint16_t)dst * inv_alpha) / 255);
}

static void draw_rgba_to_framebuffer(uint8_t* fb, size_t fb_width,
                                     size_t fb_height, size_t fb_pitch,
                                     size_t img_width, size_t img_height,
                                     const uint8_t* image_data) {
  size_t draw_width = img_width < fb_width ? img_width : fb_width;
  size_t draw_height = img_height < fb_height ? img_height : fb_height;

  for (size_t y = 0; y < draw_height; y++) {
    size_t row_start = y * img_width * 4;

    for (size_t x = 0; x < draw_width; x++) {
      const uint8_t* src = image_data + row_start + x * 4;
      uint8_t* pixel = fb + y * fb_pitch + x * 4;

      uint8_t src_r = src[0];
      uint8_t src_g = src[1];
      uint8_t src_b = src[2];
      uint8_t src_a = src[3];

      uint8_t dst_b = pixel[0];
      uint8_t dst_g = pixel[1];
      uint8_t dst_r = pixel[2];

      uint8_t out_r = blend_channel(src_r, dst_r, src_a);
      uint8_t out_g = blend_channel(src_g, dst_g, src_a);
      uint8_t out_b = blend_channel(src_b, dst_b, src_a);

      pixel[0] = out_b;
      pixel[1] = out_g;
      pixel[2] = out_r;
      pixel[3] = 0xFF;
    }
  }
}

int main(int argc, char** argv) {
  if (argc < 2) {
    fprintf(stderr, "usage: %s <image_file>\n", argv[0]);
    return EXIT_FAILURE;
  }

  const char* filename = argv[1];
  if (!filename || filename[0] == '\0') {
    fprintf(stderr, "error: no image file specified.\n");
    return EXIT_FAILURE;
  }

  // load picture
  int width, height, channels;

  uint8_t* image_data =
      stbi_load(filename, &width, &height, &channels, STBI_rgb_alpha);

  if (!image_data) {
    fprintf(stderr, "error: failed to decode %s: %s\n", filename,
            stbi_failure_reason());
    return -1;
  }

  printf("picture dimensions: %dx%d\n", width, height);
  printf("image channels: %d\n", channels);

  int fb_fd = open("/dev/fb0", O_RDWR);
  if (fb_fd < 0) {
    fprintf(stderr, "error: failed to open framebuffer device\n");
    return EXIT_FAILURE;
  }

  uint8_t* fb =
      mmap(NULL, FB_SIZE, PROT_READ | PROT_WRITE, MAP_SHARED, fb_fd, 0);

  if (fb == MAP_FAILED) {
    perror("failed to mmap /dev/fb0");
    close(fb_fd);
    stbi_image_free(image_data);
    return -1;
  }

  draw_rgba_to_framebuffer(fb, FB_WIDTH, FB_HEIGHT, FB_PITCH, (size_t)width,
                           (size_t)height, image_data);

  munmap(fb, FB_SIZE);
  close(fb_fd);
  stbi_image_free(image_data);

  return 0;
}