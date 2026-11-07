<img src="https://static.wikia.nocookie.net/vocaloid/images/a/a5/V4flower.png/revision/latest?cb=20250314003504"  height="500" align="right" style="float: right; margin: 0 10px 0 0;" >
<p align="right" style="float: right; margin: 0 10px 0 0;">Art by <a href="https://www.pixiv.net/en/users/2550807">miwashiiba</a></p>


## flower-rs (slop-edition)
a monolithic x86_64 kernel written in rust, a continuation of [riria](https://github.com/xjunko/riria).

it also natively _(somewhat)_ supports x86_64's musl ELFs.

<img width="600" height="auto" alt="image" src="https://github.com/user-attachments/assets/fbb6d10e-9565-44e4-9915-ef773d270907"  />

## what this is
```diff
- a thought experiment, just wanted to see if its possible to make a kernel in rust and also to learn rust.
+ a slop implementation of the kernel, to see the capabilities of AI.
+ this is merely a test, check-out on `master` instead if you somehow reached here.
```

## what this is not
- a good kernel, please don't use this as a reference for writing your own kernel.
- a kernel that will ever be used in production, this is just a toy project.

## building
you will need:
```
- git
- make
- qemu
- rust
- xorriso
- x86_64-linux-musl-native toolchains
```
to get a clue as to how to compile the kernel, run `make`.
```
available targets:
  - kernel // compiles the kernel
  - initrd // compiles the initrd
  - ports  // compiles the ports
  - run    // compiles the kernel and runs in qemu w/ kvm
  - clean  // deletes build artifacts
```

## things that work
### kernel
- gdt/idr/irq/isr
  - GDT INIT .... OK! /j
  - these came free with the x86_64 crates, atp this is just like building a software lol.
- pmm/vmm/paging/heap
  - works i guess, still waiting for a rewrite, but it works:tm: for now.
- vfs
  - devfs (`/dev/`), tarfs (`/init/`)
  - basic operations like open/read/write/close work, but that's it for now.
- apic/lapic
  - acpi_pmt
  - tsc
  - lapic
- pci
  - basic ac97 driver, exposed thru `/dev/audio`
    - expects 48khz of pcm stream
- scheduling
  - it works, single-thread only.
- syscalls
  - here are the supported [syscalls](abi/src/numbers.rs).

### userspace
- elf
  - it runs, no dynamic linking.
  - supports fork and execve

## things that don't work
### kernel
- vfs
  - ext2 would be nice to have
  - fat would be nice to have
  - pipe would be nice to have
- smp
  - only single core is supported

### userspace
- process
  - signals, etc.
- dynamic linking

and thousands other stuff that i don't remember or know yet.

## why(s)
### why musl instead of mlibc?
well, i did try it, mlibc is very nice to use, the problem was that, i need to set up the tooling, which is just a PITA to get it to work and portable. 
there were solutions such [[jinx]](https://github.com/mintsuki/jinx) and others but i'm too lazy to set it up. 
in the end i ended up using musl because during one of my random attempts to port libc, it just sorta works (albeit with slight syscall hacking) as is, so thats why.
sorry to the OSDEV peeps who probably think this is a very terrible idea.

## credits
these were the projects that i used for reference:
- [riria](https://github.com/xjunko/riria)
- [seele](https://github.com/SeeleOS/seele)
- [zinnia](https://github.com/zinnia-os/zinnia)
- [evalynOS](https://git.evalyngoemer.com/evalynOS/evalynOS)

## license
ISC License, see [[LICENSE]](LICENSE) for more details.
