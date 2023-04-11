#define FD_STDOUT 1

int write(int fd, const char *s, int len) {
    register int a0 asm("a0") = fd;
    register const char *a1 asm("a1") = s;
    register int a2 asm("a2") = len;
    register int ret asm("a0");

    asm volatile(
        "li a7, 64\n\t"
        "ecall\n\t"
        : "=r"(ret)
        : "r"(a0), "r"(a1), "r"(a2)
        : "a7"
    );

    return ret;
}

__attribute__((noreturn)) void exit(int code) {
    __asm__ volatile (
        "li a7, 93\n"   // system call number for exit
        "mv a0, %0\n"   // move exit code to argument 0
        "ecall\n"       // invoke the system call
        :
        : "r"(code)
        : "a0", "a7");   // clobbered registers
    while (1);
}

void main() {
    const char hello[] = "hello world\n";
    // write(FD_STDOUT, hello, sizeof(hello) - 1);
    exit(0);
}
