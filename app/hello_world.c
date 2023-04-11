#define STDOUT_FILENO 1
#define SYS_WRITE 64
#define SYS_EXIT 93

void _start() {
    const char* message = "Hello, world!\n";
    unsigned long len = 14;
    long ret_val;
    asm volatile (
        "mv a0, %1\n"      // move file descriptor to argument register a0
        "mv a1, %2\n"      // move buffer address to argument register a1
        "mv a2, %3\n"      // move buffer length to argument register a2
        "li a7, %4\n"      // set system call number to write()
        "ecall\n"          // execute write() syscall
        "mv %0, a0\n"      // move return value from a0 to ret_val variable
        "li a0, 0\n"       // set exit status to 0
        "li a7, %5\n"      // set system call number to exit()
        "ecall\n"          // execute exit() syscall
        : "=r" (ret_val)   // output operands
        : "r" (STDOUT_FILENO), "r" (message), "r" (len), "i" (SYS_WRITE), "i" (SYS_EXIT) // input operands
        : "a0", "a1", "a2", "a7" // clobbered registers
    );
}
