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

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "exec.h"
#include "input.h"

#define BUFFER_SIZE 64

static void help(const char* args) {
  (void)args;

  puts("available commands:");
  puts("  exec <filename> [args...] - fork and exec in child");
  puts("  exit - exit the shell");
  puts("  help - show this message");
}

static void exec_command(const char* input) {
  char buffer[BUFFER_SIZE];
  snprintf(buffer, sizeof(buffer), "%s", input);

  char* saveptr;
  char* cmd = strtok_r(buffer, " ", &saveptr);

  if (cmd == NULL) return;

  char* args = saveptr ? saveptr : "";

  while (*args == ' ') args++;

  if (strcmp(cmd, "help") == 0) {
    help(args);
  } else if (strcmp(cmd, "exec") == 0) {
    tools_exec_run(args);
    printf("exec: %s\n", args);
  } else if (strcmp(cmd, "exit") == 0) {
    exit(0);
  } else {
    char path[BUFFER_SIZE * 2];
    snprintf(path, sizeof(path), "/bin/%s", cmd);

    if (access(path, F_OK) == 0) {
      char command[BUFFER_SIZE * 2];
      snprintf(command, sizeof(command), "%s %s", path, args);
      tools_exec_run(command);
    } else {
      printf("unknown command: %s\n", cmd);
    }
  }
}

int main(void) {
  tools_exec_run_quiet("/bin/fetch");

  uint8_t buf[BUFFER_SIZE];

  while (true) {
    printf("> ");
    fflush(stdout);

    size_t len = read_line(buf, sizeof(buf));

    if (len == 0) continue;

    buf[len] = '\0';

    exec_command((const char*)buf);
  }
}