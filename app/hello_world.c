#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define STDERR_FILENO 1

#define SYS_READ 63
#define SYS_WRITE 64
#define SYS_EXIT 93

long sys_read(int fd, void *buf, unsigned long count){
    long ret_val;
    asm volatile (
        "mv a0, %1\n"      // move file descriptor to argument register a0
        "mv a1, %2\n"      // move buffer address to argument register a1
        "mv a2, %3\n"      // move buffer length to argument register a2
        "li a7, %4\n"      // set system call number to read()
        "ecall\n"          // execute read() syscall
        "mv %0, a0\n"      // move return value from a0 to ret_val variable
        : "=r" (ret_val)   // output operands
        : "r" (fd), "r" (buf), "r" (count), "i" (SYS_READ)  // input operands
        : "a0", "a1", "a2", "a7" // clobbered registers
    );
    return ret_val;
}

void sys_write(int fd, const char *buf, unsigned long count){
    asm volatile (
        "mv a0, %0\n"      // move file descriptor to argument register a0
        "mv a1, %1\n"      // move buffer address to argument register a1
        "mv a2, %2\n"      // move buffer length to argument register a2
        "li a7, %3\n"      // set system call number to write()
        "ecall\n"          // execute write() syscall
        :                      // output operands
        : "r" (fd), "r" (buf), "r" (count), "i" (SYS_WRITE)  // input operands
        : "a0", "a1", "a2", "a7" // clobbered registers
    );
}

void sys_exit(int status){
    asm volatile (
        "mv a0, %0\n"      // move status code to argument register a0
        "li a7, %1\n"      // set system call number to exit()
        "ecall\n"          // execute exit() syscall
        :                   // output operands
        : "r" (status), "i" (SYS_EXIT)  // input operands
        : "a0", "a7"       // clobbered registers
    );
}

void _start() {
    const char* message1 = "Hello, world!\n";
    unsigned long len1 = 14;
    const char* message2 = "Type a letter: ";
    unsigned long len2 = 15;
    char buf[3];
    int read_cnt;
    int ret_val = 0;
    sys_write(STDOUT_FILENO, message1, len1);
    sys_write(STDOUT_FILENO, message2, len2);
    read_cnt = sys_read(STDIN_FILENO, buf, 3);
    sys_write(STDOUT_FILENO, buf, read_cnt);
    sys_exit(ret_val);
}
