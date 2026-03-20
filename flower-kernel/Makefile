.PHONY: kernel
kernel:
	RUSTFLAGS="-C relocation-model=static" cargo build \
				-Zbuild-std=core,alloc \
				--target x86_64-unknown-none \
				--profile release
