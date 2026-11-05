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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/utsname.h>

void die(const char* msg) {
  perror(msg);
  exit(1);
}

struct ram_info {
  unsigned long total;
  unsigned long used;
  unsigned long free;
};

struct ram_info get_ram_info(void) {
  struct ram_info info = {0};
  char linebuf[256];

  FILE* fp;
  char* line;

  if (!(fp = fopen("/proc/meminfo", "r"))) die("fopen");

  while ((line = fgets(linebuf, sizeof(linebuf), fp))) {
    if (sscanf(line, "MemAvailable: %ld kB", &info.free)) {
      info.free *= 1024;
    } else if (sscanf(line, "MemTotal: %ld kB", &info.total)) {
      info.total *= 1024;
    }
  }
  info.used = info.total - info.free;

  fclose(fp);
  return info;
}

struct kernel_info {
  char name[64];
  char version[64];
};

struct kernel_info get_kernel_info(void) {
  struct kernel_info info = {0};
  struct utsname uts;

  if (uname(&uts) < 0) die("uname");

  strncpy(info.name, uts.sysname, sizeof(info.name) - 1);
  strncpy(info.version, uts.release, sizeof(info.version) - 1);

  return info;
}

int main() {
  struct ram_info ram = get_ram_info();
  struct kernel_info kernel = get_kernel_info();

  printf("flower@vocahuds\n");
  printf("------------\n");
  printf("kernel: %s %s\n", kernel.name, kernel.version);
  printf("memory: %lu/%lu MB\n", ram.used / (1024 * 1024),
         ram.total / (1024 * 1024));

  return 0;
}