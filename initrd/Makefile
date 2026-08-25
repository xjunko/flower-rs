SRCS  := $(filter-out Makefile %.gitkeep,$(wildcard *))
WHERE ?= /tmp/flower-initrd.tar

.PHONY: sync
sync:
	tar --blocking-factor 1 -cf $(WHERE) $(SRCS)

all: sync