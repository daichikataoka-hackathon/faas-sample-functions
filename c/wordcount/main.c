/* wordcount: count lines, words, bytes of stdin; print "lines words bytes\n". */
#include <unistd.h>
#include <stdio.h>

int main(void) {
    char buf[4096];
    long n, lines = 0, words = 0, bytes = 0;
    int in_word = 0;
    while ((n = read(0, buf, sizeof(buf))) > 0) {
        for (long i = 0; i < n; i++) {
            char c = buf[i];
            bytes++;
            if (c == '\n') lines++;
            if (c == ' ' || c == '\n' || c == '\t' || c == '\r') {
                in_word = 0;
            } else if (!in_word) {
                in_word = 1;
                words++;
            }
        }
    }
    if (n < 0) return 1;
    char out[64];
    int len = snprintf(out, sizeof(out), "%ld %ld %ld\n", lines, words, bytes);
    if (len < 0 || write(1, out, (unsigned long)len) < 0) return 1;
    return 0;
}
