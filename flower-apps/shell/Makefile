CC := x86_64-linux-musl-gcc

CFLAGS  := -O2 -Wall -Wextra \
           -fno-pie -fno-pic \
           -fno-asynchronous-unwind-tables

LDFLAGS := -static -no-pie

TARGET := ../target/x86_64-unknown-none/release
OUTPUT := $(TARGET)/flower-apps-shell

SRCS := $(shell find . -type f -name '*.c')
OBJS := $(SRCS:.c=.o)
DEPS := $(OBJS:.o=.d)

.PHONY: all clean

all: $(OUTPUT)

$(OUTPUT): $(OBJS)
	mkdir -p $(TARGET)
	$(CC) $(LDFLAGS) $^ -o $@

%.o: %.c
	$(CC) $(CFLAGS) -MMD -MP -c $< -o $@

-include $(DEPS)

clean:
	rm -f $(OBJS) $(DEPS) $(OUTPUT)