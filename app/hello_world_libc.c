#include <unistd.h>
#include <string.h>

int main() {
    const char* message = "Hello, world!\n";
    ssize_t bytes_written = write(STDOUT_FILENO, message, strlen(message));
    return 0;
}
