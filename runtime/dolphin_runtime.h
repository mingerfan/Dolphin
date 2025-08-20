#ifndef DOLPHIN_RUNTIME_C_RUNTIME_H
#define DOLPHIN_RUNTIME_C_RUNTIME_H


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <uart.h>

typedef struct {
  void *start;
  void *end;
} Area;

void ctrap(unsigned char retv);

#ifdef __cplusplus
}
#endif

#endif // RUNTIME_C_RUNTIME_H
