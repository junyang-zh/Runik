# Variables that should be passed 
TARGET ?=
PLATFORM ?=
MODE ?=
KERNEL_ELF ?=

KERNEL_BIN := $(KERNEL_ELF).bin

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
else ifeq ($(MODE), debug)
	MODE_ARG :=
endif

# Utils
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# BOARD
BOARD := qemu

build_kernel_bin: $(KERNEL_BIN)

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Target: $(TARGET)
	@echo Platform: $(BOARD)
	@RUNIK_PLATFORM=$(PLATFORM) cargo build --target $(TARGET) $(MODE_ARG)

clean:
	@cargo clean
	@rm -rf ./src/link_app.S

.PHONY: build_kernel_bin kernel clean
