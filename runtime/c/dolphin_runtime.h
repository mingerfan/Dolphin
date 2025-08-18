#ifndef DOLPHIN_RUNTIME_C_RUNTIME_H
#define DOLPHIN_RUNTIME_C_RUNTIME_H

#ifdef __cplusplus
extern "C" {
#endif

static inline void ctrap(unsigned char retv) {
  asm volatile("mv a0, %0; ebreak" : :"r"(retv));
}

#ifdef __cplusplus
}
#endif

#endif // RUNTIME_C_RUNTIME_H
