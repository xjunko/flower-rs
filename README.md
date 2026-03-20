## flower-rs
a x86_64 kernel written in rust.
<img width="1410" height="905" alt="image" src="https://github.com/user-attachments/assets/09c01cd9-ad8f-47f2-898a-8c5724e9d3a8" />


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
