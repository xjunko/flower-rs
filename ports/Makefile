export CARGO_TARGET_DIR := $(CURDIR)/target

# independent apps
APPS := shell fetch image mp3

TARGET := target/x86_64-unknown-none/release
DEST := ../initrd/initramfs/bin

.PHONY: all
all:
	@for app in $(APPS); do \
		$(MAKE) -C $$app; \
	done

	rm -f ../initrd/initramfs/.initramfs.hash
	mkdir -p $(DEST)

	@for app in $(APPS); do \
		cp $(TARGET)/flower-apps-$$app $(DEST)/$$app; \
	done
