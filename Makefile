override IMAGE_NAME := flower
override TEMP := /tmp/$(IMAGE_NAME)-build


# running
.PHONY: run
run: $(IMAGE_NAME).iso
	qemu-system-x86_64 -machine q35,accel=kvm,smm=on -s -smp 1 -m 128M \
					   -cpu host,-x2apic,+invtsc,-pdpe1gb \
                       -device e1000 -vga std -d guest_errors,int \
		               -serial stdio -no-reboot -no-shutdown \
					   -audio driver=sdl,model=ac97,id=0 \
					   -cdrom $(IMAGE_NAME).iso -d int

# kernel build
.PHONY: $(IMAGE_NAME).iso
all: $(IMAGE_NAME).iso

.PHONY: kernel
kernel:
	make -C kernel

# limine
LIMINE_ROOT := $(TEMP)/limine
$(LIMINE_ROOT)/limine:
	rm -rf $(LIMINE_ROOT)
	git clone https://github.com/Limine-Bootloader/Limine --branch=v10.x-binary --depth 1 $(TEMP)/limine
	$(MAKE) -C $(TEMP)/limine


$(IMAGE_NAME).iso: $(LIMINE_ROOT)/limine $(INITRAMFS_FILE) kernel
	rm -rf $(TEMP)/iso_root
	mkdir -p $(TEMP)/iso_root/boot

	# copy the kernel
	cp -v target/x86_64-flower/release/kernel $(TEMP)/iso_root/boot/kernel

	# limine stuff
	mkdir -p $(TEMP)/iso_root/boot/limine
	cp kernel/limine.conf $(TEMP)/iso_root/boot/limine/

	# limine important stuff
	mkdir -p $(TEMP)/iso_root/EFI/BOOT
	cp -v $(LIMINE_ROOT)/limine-bios.sys $(LIMINE_ROOT)/limine-bios-cd.bin \
		  $(LIMINE_ROOT)/limine-uefi-cd.bin $(TEMP)/iso_root/boot/limine
	cp -v $(LIMINE_ROOT)/BOOTX64.EFI $(TEMP)/iso_root/EFI/BOOT
	cp -v $(LIMINE_ROOT)/BOOTIA32.EFI $(TEMP)/iso_root/EFI/BOOT

	# final
	xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus \
		-apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		-o $(IMAGE_NAME).iso $(TEMP)/iso_root

.PHONY: clean
clean:
	cargo clean
	rm -rf $(TEMP)/iso_root $(IMAGE_NAME).iso
