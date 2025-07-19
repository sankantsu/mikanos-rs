#include <array>

// MikanOS libcxx_support depends on printk()
int printk(const char* format, ...) {
    return 0;
}

extern "C" {

// Test functions for FFI functionality

int add(int a, int b) {
    return a + b;
}

int foo() {
    std::array<int, 16> v{};
    for (int i = 0; i < 16; i++) {
        v[i] = i;
    }

    int s = 0;
    for (auto x : v) {
        s += x;
    }
    return s;
}

}
