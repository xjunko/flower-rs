<img src="https://static.wikia.nocookie.net/vocaloid/images/a/a5/V4flower.png/revision/latest?cb=20250314003504"  height="500" align="right" style="float: right; margin: 0 10px 0 0;" >
<p align="right" style="float: right; margin: 0 10px 0 0;">Art by <a href="https://www.pixiv.net/en/users/2550807">miwashiiba</a></p>


## flower-rs
a monolithic x86_64 kernel written in rust, a continuation of [riria](https://github.com/xjunko/riria).

## building
you will need:
```
- git
- make
- qemu-system-*
- rust
- xorriso
```

run `make kernel` to build the kernel. <br>
run `make run` to build the kernel and run in qemu. <br>
run `make clean` to clean build artifacts.


## credits
early part of the kernel was loosely based of [riria](https://github.com/xjunko/riria).
some part of the newer rust side of things are based on [seele](https://github.com/SeeleOS/seele).

## license
ISC License, see [[LICENSE]](LICENSE) for more details.
