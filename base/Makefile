SRCS := $(filter-out Makefile %.gitkeep,$(wildcard *))

.PHONY: sync
sync:
	tar --blocking-factor 1 -cf ../init.tar $(SRCS)

.PHONY: clean
clean:
	rm -f ../init.tar

all: sync