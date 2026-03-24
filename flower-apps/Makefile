export CARGO_TARGET_DIR := $(CURDIR)/target

.PHONY: all
all:
	$(MAKE) -C shell
	$(MAKE) -C fetch
	$(MAKE) -C cat
	$(MAKE) -C echo
	$(MAKE) -C hello
	$(MAKE) -C pcm
	$(MAKE) -C png

	rm -f ../flower-boot/initramfs/.initramfs.hash
	mkdir -p ../flower-boot/initramfs/bin
	cp target/x86_64-unknown-none/release/flower-apps-shell ../flower-boot/initramfs/bin/shell
	cp target/x86_64-unknown-none/release/flower-apps-fetch ../flower-boot/initramfs/bin/fetch
	cp target/x86_64-unknown-none/release/flower-apps-cat   ../flower-boot/initramfs/bin/cat
	cp target/x86_64-unknown-none/release/flower-apps-png   ../flower-boot/initramfs/bin/png
	cp target/x86_64-unknown-none/release/flower-apps-echo  ../flower-boot/initramfs/bin/echo
	cp target/x86_64-unknown-none/release/flower-apps-hello ../flower-boot/initramfs/bin/hello
	cp target/x86_64-unknown-none/release/flower-apps-pcm   ../flower-boot/initramfs/bin/pcm
