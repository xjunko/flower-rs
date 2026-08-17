export CARGO_TARGET_DIR := $(CURDIR)/target

APPS := shell unix
UNIX_APPS := cat echo sleep wav png fetch hello
TARGET := target/x86_64-unknown-none/release
DEST := ../base/usr/bin

.PHONY: all
all:
	@for app in $(APPS); do \
		$(MAKE) -C $$app; \
	done

	mkdir -p $(DEST)

	@for app in $(APPS); do \
		cp $(TARGET)/flower-apps-$$app $(DEST)/$$app; \
	done

	# busybox-like
	@for app in $(UNIX_APPS); do \
		cp $(TARGET)/flower-apps-unix $(DEST)/$$app; \
	done
