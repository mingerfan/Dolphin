#include <dolphin_runtime.h>
#include <klib-macros.h>
#include <device_config.h>

#ifndef MAINARGS
#define MAINARGS ""
#endif
static const char mainargs[] = MAINARGS;

extern char _heap_start;
Area heap = RANGE(&_heap_start, MEMORY_BASE + MEMORY_SIZE * 1024 * 1024);

int main(const char *args);

void _trm_init() {
    int ret = main(mainargs);
    ctrap(ret);
}
