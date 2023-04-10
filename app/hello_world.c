#define SYS_WRITE 64

typedef unsigned long size_t;
typedef long ssize_t;

ssize_t write(int fd, const void *buf, size_t count) {
  ssize_t ret = -1;

  __asm__ volatile (
    "li a7, 64\n"         // system call number for write
    "mv a0, %1\n"         // move file descriptor to argument 0
    "mv a1, %2\n"         // move buffer address to argument 1
    "mv a2, %3\n"         // move length to argument 2
    "ecall\n"             // invoke the system call
    "mv %0, a0\n"         // move the return value to 'ret'
    : "=r"(ret)           // output operand (ret)
    : "r"(fd), "r"(buf), "r"(count)  // input operands
    : "a0", "a1", "a2", "a7");       // clobbered registers

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
}

void _start() {
  const char hello[] = "hello world\n";
  write(SYS_WRITE, hello, sizeof(hello) - 1);
  exit(0);
}
