/* Minimal FaaS handler (C): read all of stdin, uppercase it, write to stdout. */
#include <unistd.h>
#include <ctype.h>

int main(void) {
    char buf[4096];
    long n;
    while ((n = read(0, buf, sizeof(buf))) > 0) {
        for (long i = 0; i < n; i++) {
            buf[i] = (char)toupper((unsigned char)buf[i]);
        }
        if (write(1, buf, (unsigned long)n) < 0) {
            return 1;
        }
    }
    return n < 0 ? 1 : 0;
}
