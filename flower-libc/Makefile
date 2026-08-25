MODE ?= release

.PHONY: all
all:
	RUSTFLAGS="-C relocation-model=static" cargo build --profile release $(if $(filter debug,$(MODE)),--features debug,)
