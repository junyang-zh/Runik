.PHONY: build clean

build:
	@mkdir -p target
	@riscv64-unknown-elf-gcc hello_world_libc.c -o target/hello_world.bin
	@echo "App hello_world built!"

clean:
	@rm -rf target