MODE ?= release

CC := clang
RUST_TARGET ?= x86_64-unknown-none
CLANG_TARGET ?= x86_64-unknown-none-elf
ROOT := ../..
LIBC := $(ROOT)/target/$(RUST_TARGET)/$(if $(filter debug,$(MODE)),debug,release)/libflower_libc.a

CFLAGS := --target=$(CLANG_TARGET) -ffreestanding -fno-builtin -fno-stack-protector \
	-fno-pie -fno-asynchronous-unwind-tables -I$(ROOT)/flower-libc/include \
	-O$(if $(filter debug,$(MODE)),0,3)
LDFLAGS := --target=$(CLANG_TARGET) -nostdlib -nostartfiles -nodefaultlibs -static \
	-fuse-ld=lld -Wl,-T,link.ld

.PHONY: all clean libc

all: c-example

libc:
	$(MAKE) -C $(ROOT)/flower-libc MODE=$(MODE)

main.o: main.c $(ROOT)/flower-libc/include/flower.h
	$(CC) $(CFLAGS) -c $< -o $@

c-example: libc main.o link.ld
	$(CC) $(LDFLAGS) main.o $(LIBC) -o $@

clean:
	rm -f main.o c-example
