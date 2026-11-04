CC := x86_64-linux-musl-gcc

CFLAGS  := -fno-pie -fno-pic -fno-asynchronous-unwind-tables
LDFLAGS := -static -no-pie

TARGET := ../target/x86_64-unknown-none/release

.PHONY: all clean

all: flower-apps-musl


main.o: main.c
	$(CC) $(CFLAGS) -c $< -o $@

flower-apps-musl: main.o
	$(CC) $(LDFLAGS) main.o $(LIBC) -o $(TARGET)/$@

clean:
	rm -f main.o flower-apps-musl
