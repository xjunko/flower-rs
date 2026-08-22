.PHONY: kernel
kernel:
	cargo build --profile release

.PHONY: clean
clean:
	cargo clean

<<<<<<< HEAD
# running
.PHONY: run
run: ../$(IMG_NAME).iso
	qemu-system-x86_64 -cpu host -machine q35,accel=kvm -smp 1 -m 128M \
                       -device e1000 -vga std -d guest_errors,int \
		               -serial stdio -no-reboot -no-shutdown \
					   -audio driver=sdl,model=ac97,id=0 \
					   -cdrom ../$(IMG_NAME).iso 
=======
all: kernel
>>>>>>> 3b160d1 (makefile(s): remove unused parts)
