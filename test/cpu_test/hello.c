#include <dolphin_runtime.h>

int main() {
    int sum = 0;
    for (int i = 1; i <= 100; ++i) {
        sum += i;
    }
    ctrap(0);
}
