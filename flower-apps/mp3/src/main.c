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

#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define DR_MP3_NO_SIMD
#define DR_MP3_IMPLEMENTATION
#include "thirdparty/dr_mp3.h"

#define CHUNK_SIZE 4096

static uint64_t current_time_ms(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000 + (uint64_t)ts.tv_nsec / 1000000;
}

static void sleep_ms(uint64_t ms) {
  struct timespec ts = {.tv_sec = ms / 1000, .tv_nsec = (ms % 1000) * 1000000};
  while (nanosleep(&ts, &ts) == -1 && errno == EINTR);
}

int main(int argc, char** argv) {
  if (argc < 2) {
    fprintf(stderr, "usage: %s <mp3_file>\n", argv[0]);
    return EXIT_FAILURE;
  }

  const char* filename = argv[1];
  if (!filename || filename[0] == '\0') {
    fprintf(stderr, "error: no MP3 file specified.\n");
    return EXIT_FAILURE;
  }

  // driver
  int driver_fd = open("/dev/audio", O_WRONLY);
  if (driver_fd < 0) {
    perror("failed to open audio driver");
    return EXIT_FAILURE;
  }

  // decoder
  drmp3 mp3;
  if (!drmp3_init_file(&mp3, filename, NULL)) {
    fprintf(stderr, "failed to initialize MP3 stream.\n");
    close(driver_fd);
    return EXIT_FAILURE;
  }

  // info
  printf(
      "playing MP3: %s\n"
      "channels: %u\n"
      "sample rate: %u Hz\n",
      filename, mp3.channels, mp3.sampleRate);

  // buffers
  drmp3_int16 pcm_buffer[CHUNK_SIZE * 2];
  drmp3_uint64 total_frames_decoded = 0;

  uint64_t next_deadline = current_time_ms();
  int ret = EXIT_SUCCESS;

  for (;;) {
    drmp3_uint64 frames_decoded =
        drmp3_read_pcm_frames_s16(&mp3, CHUNK_SIZE, pcm_buffer);

    if (frames_decoded == 0) break;
    total_frames_decoded += frames_decoded;

    size_t bytes_to_write =
        (size_t)frames_decoded * mp3.channels * sizeof(drmp3_int16);
    size_t written_total = 0;

    while (written_total < bytes_to_write) {
      ssize_t written = write(driver_fd, (uint8_t*)pcm_buffer + written_total,
                              bytes_to_write - written_total);

      if (written < 0) {
        if (errno == EINTR) continue;
        perror("failed to write to audio driver");
        ret = EXIT_FAILURE;
        goto cleanup;
      }

      if (written == 0) {
        fprintf(stderr, "audio driver returned zero bytes\n");
        ret = EXIT_FAILURE;
        goto cleanup;
      }
      written_total += (size_t)written;
    }

    // playback duration
    uint64_t chunk_duration_ms = (frames_decoded * 1000) / mp3.sampleRate;
    next_deadline += chunk_duration_ms;

    // now we wait....
    uint64_t now = current_time_ms();
    if (next_deadline > now) {
      sleep_ms(next_deadline - now);
    }

    printf("\rprogress: %llu frames decoded, %llu total", frames_decoded,
           total_frames_decoded);
  }

  printf("\nplayback finished.\n");

cleanup:
  drmp3_uninit(&mp3);
  close(driver_fd);

  return ret;
}