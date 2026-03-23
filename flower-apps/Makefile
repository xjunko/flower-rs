export CARGO_TARGET_DIR := $(CURDIR)/target

.PHONY: all
all:
	$(MAKE) -C shell
	$(MAKE) -C cat
	$(MAKE) -C hello
	$(MAKE) -C pcm

	rm -f ../flower-boot/initramfs/.initramfs.hash
	cp target/x86_64-unknown-none/release/flower-apps-shell ../flower-boot/initramfs/shell
	cp target/x86_64-unknown-none/release/flower-apps-cat   ../flower-boot/initramfs/cat
	cp target/x86_64-unknown-none/release/flower-apps-hello ../flower-boot/initramfs/hello
	cp target/x86_64-unknown-none/release/flower-apps-pcm   ../flower-boot/initramfs/pcm

