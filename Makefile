##### compilation targets #####

.PHONY: default
default:
	@echo "available targets:"
	@echo "  - kernel // compiles the kernel"
	@echo "  - run    // compiles the kernel and runs in qemu w/ kvm"
	@echo "  - clean  // deletes build artifacts"

# kernel doesnt rely on anything, it can be built by itself
.PHONY: kernel
kernel:
	$(MAKE) -C kernel

##### image generation #####
override IMG_NAME := flower
override TMP := /tmp/$(IMG_NAME)-build

.PHONY: $(IMG_NAME).iso
all: $(IMG_NAME).iso

# limine
override LIMINE := $(TMP)/limine
$(LIMINE)/limine:
	rm -rf $(LIMINE)
	git clone https://github.com/Limine-Bootloader/Limine --branch=v11.x-binary --depth 1 $(LIMINE)
	$(MAKE) -C $(LIMINE)

# compiling the entire thing 
$(IMG_NAME).iso: $(LIMINE)/limine kernel 
	rm    -rf $(TMP)/iso_root
	mkdir -p  $(TMP)/iso_root/boot

	# kernel elf
	cp -v kernel/target/x86_64-flower/release/kernel $(TMP)/iso_root/boot/kernel

	# limine
	mkdir -p $(TMP)/iso_root/boot/limine
	cp    kernel/limine.conf $(TMP)/iso_root/boot/limine/

	# limine binaries
	mkdir -p $(TMP)/iso_root/EFI/BOOT

	cp    -v $(LIMINE)/limine-bios.sys $(LIMINE)/limine-bios-cd.bin \
			 $(LIMINE)/limine-uefi-cd.bin \
			 $(TMP)/iso_root/boot/limine

	cp    -v $(LIMINE)/BOOTX64.EFI $(TMP)/iso_root/EFI/BOOT
	cp    -v $(LIMINE)/BOOTIA32.EFI $(TMP)/iso_root/EFI/BOOT

	# create iso
	xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table -hfsplus  \
		-apm-block-size 2048 --efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		-o $(IMG_NAME).iso $(TMP)/iso_root

##### utils and other bs #####
.PHONY: clean
clean:
	rm -rf $(TMP)
	make -C kernel clean

.PHONY: run
run: $(IMG_NAME).iso
	qemu-system-x86_64 -machine q35,accel=kvm,smm=on -s -smp 1 -m 128M \
					   -cpu host,-x2apic,+invtsc,-pdpe1gb \
                       -device e1000 -vga std -d guest_errors,int \
		               -serial stdio -no-reboot -no-shutdown \
					   -audio driver=sdl,model=ac97,id=0 \
					   -cdrom $(IMG_NAME).iso -d int