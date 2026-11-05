##### compilation targets #####

.PHONY: default
default:
	@echo "available targets:"
	@echo "  - kernel // compiles the kernel"
	@echo "  - initrd // compiles the initrd"
	@echo "  - ports  // compiles the ports"
	@echo "  - run    // compiles the kernel and runs in qemu w/ kvm"
	@echo "  - clean  // deletes build artifacts"

# targets
.PHONY: ports
ports:
	make -C ports

.PHONY: initrd
initrd: ports
	$(MAKE) -C initrd/initramfs

.PHONY: kernel
kernel: initrd ports
	make -C kernel

##### image generation #####
override IMAGE_NAME := flower
override TEMP := /tmp/$(IMAGE_NAME)-build

.PHONY: $(IMAGE_NAME).iso
all: $(IMAGE_NAME).iso

# limine
override LIMINE := $(TEMP)/limine
$(LIMINE)/limine:
	rm -rf $(LIMINE)
	git clone https://github.com/Limine-Bootloader/Limine --branch=v11.x-binary --depth 1 $(LIMINE)
	$(MAKE) -C $(LIMINE)

# the entire thing
$(IMAGE_NAME).iso: $(LIMINE)/limine kernel initrd
	rm -rf $(TEMP)/iso_root
	mkdir -p $(TEMP)/iso_root/boot

	# copy the kernel
	cp -v target/x86_64-unknown-none/release/flower-kernel $(TEMP)/iso_root/boot/kernel

	# copy initramfs
	cp -v /tmp/flower-initrd.tar $(TEMP)/iso_root/boot/initrd

	# limine stuff
	mkdir -p $(TEMP)/iso_root/boot/limine
	cp initrd/limine.conf $(TEMP)/iso_root/boot/limine/

	# limine important stuff
	mkdir -p $(TEMP)/iso_root/EFI/BOOT
	cp -v $(LIMINE)/limine-bios.sys $(LIMINE)/limine-bios-cd.bin \
		  $(LIMINE)/limine-uefi-cd.bin $(TEMP)/iso_root/boot/limine
	cp -v $(LIMINE)/BOOTX64.EFI $(TEMP)/iso_root/EFI/BOOT
	cp -v $(LIMINE)/BOOTIA32.EFI $(TEMP)/iso_root/EFI/BOOT

	# final
	xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
		-apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		-o $(IMAGE_NAME).iso $(TEMP)/iso_root

.PHONY: clean
clean:
	cargo clean
	make -C initrd/initramfs clean
	rm -rf $(TEMP)/iso_root $(IMAGE_NAME).iso

# running
.PHONY: run
run: $(IMAGE_NAME).iso
	qemu-system-x86_64 -cpu host -machine q35,accel=kvm -smp 2 -m 256M \
                       -device e1000 -vga std -d guest_errors,int \
		               -serial stdio -no-reboot -no-shutdown \
					   -audio driver=sdl,model=ac97,id=0 \
					   -cdrom $(IMAGE_NAME).iso -d int