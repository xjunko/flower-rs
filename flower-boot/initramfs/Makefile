OUT_TAR := ../initramfs.tar
HASH_FILE := .initramfs.hash

SRCS := $(filter-out Makefile,$(wildcard *))

.PHONY: all clean
all: $(OUT_TAR)

$(OUT_TAR): $(HASH_FILE)
	@echo "initramfs unchanged, skipping rebuild"

$(HASH_FILE): $(SRCS)
	@echo "Checking initramfs contents..."
	@tar -cf - $(SRCS) | sha256sum | cut -d' ' -f1 > $@.new
	@if [ ! -f $@ ] || ! cmp -s $@ $@.new; then \
		echo "initramfs changed, rebuilding tar"; \
		mv $@.new $@; \
		tar --blocking-factor 1 -cf $(OUT_TAR) $(SRCS); \
	else \
		rm $@.new; \
	fi

clean:
	rm -f $(HASH_FILE) $(OUT_TAR)
