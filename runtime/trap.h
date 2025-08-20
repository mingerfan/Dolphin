#ifndef TRAP_H
#define TRAP_H

#include <dolphin_runtime.h>
#include <klib.h>
#include <stdbool.h>
#include <klib-macros.h>
#include <stdint.h>


__attribute__((noinline))
void check(bool cond) {
  if (!cond) ctrap(1);
}

#endif // TRAP_H
