.PHONY: all
all:
	ccc x86_64-elf gcc -ffreestanding -O2 -Wall -Wextra -nostdlib -c hello.c -o userspace-hello-c.o
	ccc x86_64-elf ld -T link.ld -o userspace-hello-c userspace-hello-c.o
	mv userspace-hello-c ../../target/x86_64-unknown-none/release/hello-c
	rm userspace-hello-c.o