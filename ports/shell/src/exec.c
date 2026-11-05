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

#include "exec.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

static void run_inner(const char* args, int print_exit_status) {
  if (args == NULL) {
    puts("usage: exec <filename> [args...]");
    return;
  }

  // Trim leading whitespace.
  while (*args == ' ' || *args == '\t' || *args == '\n' || *args == '\r') {
    args++;
  }

  if (*args == '\0') {
    puts("usage: exec <filename> [args...]");
    return;
  }

  // Tokenize arguments.
  char* input = strdup(args);
  if (input == NULL) return;

  char* argv[64];
  size_t argc = 0;

  char* saveptr;
  char* token = strtok_r(input, " \t\n\r", &saveptr);

  while (token != NULL && argc < 63) {
    argv[argc++] = token;
    token = strtok_r(NULL, " \t\n\r", &saveptr);
  }

  argv[argc] = NULL;

  if (argc == 0) {
    free(input);
    puts("usage: exec <filename> [args...]");
    return;
  }

  pid_t pid = fork();

  if (pid < 0) {
    printf("fork failed: %d\n", pid);
    free(input);
    return;
  }

  if (pid == 0) {
    execve(argv[0], argv, NULL);

    // Only reached if execve fails.
    printf("execve failed: %d\n", -1);
    _exit(127);
  }

  int status;
  pid_t result = waitpid(pid, &status, 0);

  if (result < 0) {
    puts("waitpid failed");
    free(input);
    return;
  }

  if (print_exit_status) {
    int exit_code;

    if (WIFEXITED(status)) {
      exit_code = WEXITSTATUS(status);
    } else if (WIFSIGNALED(status)) {
      exit_code = 128 + WTERMSIG(status);
    } else {
      exit_code = status;
    }

    printf("process exited with code %d\n", exit_code);
  }

  free(input);
}

void tools_exec_run(const char* args) { run_inner(args, 1); }

void tools_exec_run_quiet(const char* args) { run_inner(args, 0); }