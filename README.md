<img src="https://static.wikia.nocookie.net/vocaloid/images/a/a5/V4flower.png/revision/latest?cb=20250314003504"  height="500" align="right" style="float: right; margin: 0 10px 0 0;" >
<p align="right" style="float: right; margin: 0 10px 0 0;">Art by <a href="https://www.pixiv.net/en/users/2550807">miwashiiba</a></p>


## flower-rs [barebones]
a monolithic x86_64 kernel written in rust, a continuation of [riria](https://github.com/xjunko/riria).

this is the [[barebones]](#) branch of the project.


<img width="600" height="auto" alt="image" src="https://github.com/user-attachments/assets/fbb6d10e-9565-44e4-9915-ef773d270907"  />

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

## what's the difference from the master branch
- this only has the kernel
- no userspace
- no libc
- no syscall

## why
this can be a good starting point to test my other ideas on making an kernel

## license
ISC License, see [[LICENSE]](LICENSE) for more details.
