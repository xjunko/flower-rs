.PHONY: all
all:
	RUSTFLAGS="-C relocation-model=static" cargo build --target x86_64-unknown-none --profile release
# 	cbindgen --config cbindgen.toml --crate flower-libc -o include/flower-libc.h
