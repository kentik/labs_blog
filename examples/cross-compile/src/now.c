#include <stddef.h>
#include <time.h>

size_t now(char *ptr, size_t len) {
    time_t now = time(NULL);
    struct tm *tm = localtime(&now);
    return strftime(ptr, len, "%F %T", tm);
}
