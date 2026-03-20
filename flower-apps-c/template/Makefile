.PHONY: all
all:
	ccc x86_64-elf gcc -ffreestanding -O2 -Wall -Wextra -nostdlib -c template.c -o obj.o
	ccc x86_64-elf ld -T link.ld -o template-c obj.o
	mv template-c ../../target/x86_64-unknown-none/release/hello-c
	rm obj.o
