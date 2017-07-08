name := micro

arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/$(name)-$(arch).iso

linker_script = src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

target ?= $(arch)-unknown-linux-gnu
os := target/$(target)/debug/libzinc_os.a

qemu_debug := qemu.log

.PHONY: all clean run iso

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

debug: $(iso)
	@qemu-system-x86_64 -d int -no-reboot -cdrom $(iso) -s -S

gdb: $(iso)
	@qemu-system-x86_64 -d int -no-reboot -cdrom $(iso) -s -S &> $(qemu_debug) &
	@rust-gdb "build/kernel-x86_64.bin" -ex "target remote localhost:1234"

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/iso/boot/grub
	@cp $(kernel) build/iso/boot/kernel.bin
	@cp $(grub_cfg) build/iso/boot/grub
	@grub-mkrescue -o $(iso) build/iso 2> /dev/null
	@rm -r build/iso

$(kernel): cargo $(os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) \
		$(os)

cargo:
	@cargo build --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
