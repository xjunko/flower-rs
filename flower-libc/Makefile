MODE ?= release

.PHONY: all
all:
	RUSTFLAGS="-C relocation-model=static" cargo build --target x86_64-unknown-none --profile release $(if $(filter debug,$(MODE)),--features debug,)
# 	cbindgen --config cbindgen.toml --crate flower-libc -o include/flower-libc.h
