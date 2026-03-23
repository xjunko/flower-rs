## flower-rs
a monolithic x86_64 kernel written in rust, a continuation of [riria](https://github.com/xjunko/riria).
<img width="1410" height="905" alt="image" src="https://github.com/user-attachments/assets/3a9f8d18-f1b8-4067-b6de-f14f1493e4a8" />

## what this is
a thought experiment, just wanted to see if its possible to make a kernel in rust and also to learn rust.

## what this is not
- a good kernel, please don't use this as a reference for writing your own kernel.
- a kernel that will ever be used in production, this is just a toy project.

## building
you will need:
```
- git
- make
- qemu-system-*
- rust
- xorriso
```
to build the kernel:
```
make
```
to run it:
```
make run
```
if something breaks down for no reason then it's better to do:
```
make clean run
```

## things that work
### kernel
- gdt/idr/irq/isr
  - GDT INIT .... OK! /j
  - these came free with the x86_64 crates, atp this is just like building a software lol.
- pmm/vmm/paging/heap
  - works i guess, heap is static though.
- vfs
  - devfs (`/dev/`), tarfs (`/init/`)
  - basic operations like open/read/write/close work, but that's it for now.
- apic/lapic
  - i have timer working, but that's about it.
- pci
  - super basic ac97 driver, it works and is exposed thru `/dev/audio`.
- scheduling
  - it works.
- syscalls
  - exit, open, close, read, write, mmap, write_fs_base.
  - will add more when i start porting userland programs.

### userspace
- elf
  - it runs, no dynamic linking.
  - supports fork and execve
- programs:
  - `cat`, `echo`, `hello`, `pcm`, `shell`

## things that don't work
### kernel
- vfs
  - ext2 would be nice to have
  - fat would be nice to have
  - pipe would be nice to have
- smp
  - only single core is supported
- stability
  - sometime it got stuck on boot, so not that stable.
  
### userspace
- process
  - signals, etc.
- dynamic linking

and thousands other stuff that i don't remember or know yet.

## license
ISC License, see [[LICENSE]](LICENSE) for more details.
