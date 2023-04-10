# Docker image name
DOCKER_IMG_NAME ?= runik_dev

# Building
TARGET := riscv64gc-unknown-none-elf
PLATFORM := qemu
MODE := debug
KERNEL_ELF_RELATIVE := target/$(TARGET)/$(MODE)/runik

# Running
BOOTLOADER_URL = https://github.com/rustsbi/rustsbi-qemu/releases/download/v0.1.1/rustsbi-qemu-release.zip
BOOTLOADER = bootloader/rustsbi-qemu.bin

# Debugging
KERNEL_ELF := runik/$(KERNEL_ELF_RELATIVE)
KERNEL_BIN := $(KERNEL_ELF).bin
KERNEL_ENTRY_PA := 0x80200000
DISASM_TMP := runik/target/$(TARGET)/$(MODE)/asm

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64

# Disassembly
DISASM ?= -x

.PHONY: docker_run docker_build build clean bootloader disasm disasm-vim run-inner gdbserver gdbclient

# Build commands

build:
	@make -C app build
	@make -C runik \
		TARGET=$(TARGET) \
		PLATFORM=$(PLATFORM) \
		MODE=$(MODE) \
		KERNEL_ELF=$(KERNEL_ELF_RELATIVE) \
		build_kernel_bin

clean:
	@make -C user clean
	@make -C runik clean

# Docker commands

docker_run:
	docker run --rm -it -v `pwd`:/mnt -w /mnt ${DOCKER_IMG_NAME} bash

docker_build:
	docker build -t ${DOCKER_IMG_NAME} .

# Getting a precompiled bootloader
bootloader: $(BOOTLOADER)

$(BOOTLOADER):
	@mkdir -p bootloader
	@cd bootloader && wget $(BOOTLOADER_URL) && unzip rustsbi-qemu-release.zip

# Utility commands

disasm:
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim:
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

run: run-inner

run-inner: bootloader
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

debug: bootloader
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: bootloader
	@qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
