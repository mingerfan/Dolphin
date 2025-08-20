#include <dolphin_runtime.h>

int main();

void _trm_init() {
  int ret = main();
  ctrap(ret);
}