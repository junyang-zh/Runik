.PHONY: build

build:
	@mkdir -p target
	@clang --target=riscv64 -march=rv64gc -I/usr/local/riscv64-unknown-elf/include -L/usr/local/riscv64-unknown-elf/lib -L/usr/lib hello_world.c -o target/hello_world.bin -Tlinker.ld
	#@riscv64-unknown-linux-gnu-gcc hello_world.c -o target/hello_world.bin -Tlinker.ld
