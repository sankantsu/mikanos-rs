#include <stdio.h>
#include <stdarg.h>
#include "logger.hpp"

extern "C" void mikanos_rs_serial_print(const char* s);

namespace {
  LogLevel log_level = kInfo;
}

int Log(LogLevel level, const char* format, ...) {
    if (level > log_level) {
        return 0;
    }

    va_list ap;
    int result;
    char s[1024];

    va_start(ap, format);
    result = vsprintf(s, format, ap);
    va_end(ap);

    mikanos_rs_serial_print(s);
    return result;
}
