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

#include "input.h"

#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

uint8_t getch(void) {
  int kb = open("/dev/keyboard", O_RDONLY);

  if (kb < 0) return 0;

  uint8_t c = 0;

  while (1) {
    ssize_t ret = read(kb, &c, 1);

    if (ret < 0) {
      close(kb);
      return 0;
    }

    if (ret > 0 && c != 0) {
      close(kb);
      return c;
    }
  }
}

size_t read_line(uint8_t* buf, size_t len) {
  size_t pos = 0;

  while (1) {
    uint8_t c = getch();

    switch (c) {
      case '\n':
        putchar('\n');
        return pos;

      case '\b':
        if (pos > 0) {
          pos--;
          printf("\b \b");
          fflush(stdout);
        }
        break;

      default:
        if (c >= 32 && c <= 126) {
          if (pos < len) {
            buf[pos++] = c;
            putchar(c);
            fflush(stdout);
          }
        } else {
          return 0;
        }
        break;
    }
  }
}