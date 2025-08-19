/* C implementation of runtime functions used by both C and Rust.
   Provide a non-inline symbol for ctrap so Rust's extern can link to it. */

#include "dolphin_runtime.h"

void ctrap(unsigned char retv) {
    /* Move retv into a0 and trigger an ebreak. */
    asm volatile("mv a0, %0; ebreak" : : "r"(retv));
}
