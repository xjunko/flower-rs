#ifndef FLOWER_H
#define FLOWER_H

#include <stddef.h>
#include <stdint.h>

typedef int64_t flower_ssize_t;
typedef int64_t flower_off_t;

typedef struct {
    uint16_t st_mode;
    uint64_t st_dev;
    uint64_t st_size;
} flower_stat_t;

enum {
    FLOWER_PROT_NONE = 0,
    FLOWER_PROT_READ = 1,
    FLOWER_PROT_WRITE = 2,
    FLOWER_PROT_EXEC = 4,
};

enum {
    FLOWER_MAP_SHARED = 1,
    FLOWER_MAP_PRIVATE = 2,
    FLOWER_MAP_FIXED = 16,
    FLOWER_MAP_ANONYMOUS = 32,
};

flower_ssize_t read(uint64_t fd, void *buffer, size_t length);
flower_ssize_t write(uint64_t fd, const void *buffer, size_t length);
int64_t open(const char *path, uint64_t flags, uint64_t mode);
int64_t close(uint64_t fd);
int64_t metadata(uint64_t fd, flower_stat_t *stat);

int64_t fork(void);
int64_t waitpid(uint64_t pid);
int64_t execve(const char *path, const char *const *argv,
               const char *const *envp);
void exit(uint64_t status);

void *mmap(void *address, size_t size, uint64_t prot, uint64_t flags,
           uint64_t fd, size_t offset);
int64_t munmap(void *address, size_t size);
void sleep(uint64_t milliseconds);

#endif
