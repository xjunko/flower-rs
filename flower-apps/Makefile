export CARGO_TARGET_DIR := $(CURDIR)/target

APPS := shell fetch cat png echo hello wav
TARGET := target/x86_64-unknown-none/release
DEST := ../flower-boot/initramfs/bin

.PHONY: all
all:
	@for app in $(APPS); do \
		$(MAKE) -C $$app; \
	done

	rm -f ../flower-boot/initramfs/.initramfs.hash
	mkdir -p $(DEST)

	@for app in $(APPS); do \
		cp $(TARGET)/flower-apps-$$app $(DEST)/$$app; \
	done
