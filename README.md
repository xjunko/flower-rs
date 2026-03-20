## flower-rs
a x86_64 kernel written in rust.

<img width="770" height="665" alt="image" src="https://github.com/user-attachments/assets/035fcc1a-0b85-4a70-b002-cbffa3b30985" />

## why
thought experiment, just wanted to see if its possible.

## what works
- gdt
- idt
- pmm
- vmm
- heap
- apic
- lapic
- kernel/userspace scheduling
- basic syscall

## building
you will need:
```
- git
- qemu-system-*
- rust
- xorriso
```
to build the kernel just run
```
make
```
to run it
```
make run
```
